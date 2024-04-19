use crate::configuration::Configuration;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::Client;
use log::debug;
use security_framework::base::Error;
use security_framework::passwords::get_generic_password;
use std::env::VarError;
use std::string::FromUtf8Error;
use std::{env, fmt};
use whoami;

#[derive(Debug, Eq, Hash, PartialEq)]
enum EnvVar {
    AwsAccessKeyId,
    AwsSecretAccessKey,
    AwsDefaultRegion,
}

impl From<EnvVar> for &str {
    fn from(key: EnvVar) -> Self {
        match key {
            EnvVar::AwsAccessKeyId => "AWS_ACCESS_KEY_ID",
            EnvVar::AwsSecretAccessKey => "AWS_SECRET_ACCESS_KEY",
            EnvVar::AwsDefaultRegion => "AWS_DEFAULT_REGION",
        }
    }
}

enum KeychainServices {
    AwsAccessKeyId,
    AwsSecretAccessKey,
}

impl From<KeychainServices> for &str {
    fn from(key: KeychainServices) -> Self {
        match key {
            KeychainServices::AwsAccessKeyId => "com.logga.aws-access-key-id",
            KeychainServices::AwsSecretAccessKey => "com.logga.aws-secret-access-key",
        }
    }
}

#[derive(Debug)]
pub enum ClientError<'a> {
    CredentialNotSet(&'a str, VarError),
    KeychainReadFailed(Error),
    KeychainPasswordParseFailed(std::string::FromUtf8Error),
}

impl fmt::Display for ClientError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::CredentialNotSet(key, err) => {
                write!(f, "{} {}", key, err)
            }
            ClientError::KeychainReadFailed(err) => write!(f, "{}", err),
            ClientError::KeychainPasswordParseFailed(err) => write!(f, "{}", err),
        }
    }
}

impl From<Error> for ClientError<'_> {
    fn from(err: Error) -> Self {
        match err {
            err => ClientError::KeychainReadFailed(err),
        }
    }
}

impl From<FromUtf8Error> for ClientError<'_> {
    fn from(err: FromUtf8Error) -> Self {
        match err {
            err => ClientError::KeychainPasswordParseFailed(err),
        }
    }
}

pub async fn create_s3_client(config: &Configuration) -> Result<Client, ClientError> {
    let region = env_var(EnvVar::AwsDefaultRegion).unwrap_or_else(|_| config.s3.region.clone());
    let region_provider = Region::new(region);

    let credentials_store = get_aws_credentials(&config)?;

    let credentials = Credentials::new(
        credentials_store.0,
        credentials_store.1,
        None,
        None,
        "s3_compatible_backend",
    );

    let region_provider = RegionProviderChain::default_provider().or_else(region_provider);
    let shared_config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(region_provider)
        .credentials_provider(credentials)
        .load()
        .await;
    let shared_config_with_endpoint = aws_sdk_s3::config::Builder::from(&shared_config)
        .endpoint_url(&config.s3.endpoint)
        .build();

    Ok(Client::from_conf(shared_config_with_endpoint))
}

fn env_var<'a>(key: EnvVar) -> Result<String, ClientError<'a>> {
    let key_str: &str = key.into();
    env::var(key_str).map_err(|e| ClientError::CredentialNotSet(key_str, e))
}

fn keychain_item<'a>(key: KeychainServices, user: &String) -> Result<String, ClientError<'a>> {
    let password = get_generic_password(key.into(), &user)?;
    Ok(String::from_utf8(password)?)
}

#[derive(Debug)]

struct CredentialsStore(String, String);

fn get_aws_credentials<'a>(config: &Configuration) -> Result<CredentialsStore, ClientError<'a>> {
    if config.s3.keychain_authentication {
        debug!("Trying to read AWS credentials from Keychain.");

        let user = whoami::username();
        let access_key_id = keychain_item(KeychainServices::AwsAccessKeyId.into(), &user)?;
        let secret_key = keychain_item(KeychainServices::AwsSecretAccessKey, &user)?;
        return Ok(CredentialsStore(access_key_id, secret_key));
    }

    Ok(CredentialsStore(
        env_var(EnvVar::AwsAccessKeyId)?,
        env_var(EnvVar::AwsSecretAccessKey)?,
    ))
}

#[cfg(test)]
mod tests {
    use std::env;

    use security_framework::passwords::{delete_generic_password, set_generic_password};

    use crate::{
        configuration::{Configuration, S3},
        s3_client::{self, ClientError, CredentialsStore, KeychainServices},
    };

    use super::EnvVar;

    macro_rules! assert_err {
        ($expression:expr, $($pattern:tt)+) => {
            match $expression {
                $($pattern)+ => (),
                ref e => panic!("expected `{}` but got `{:?}`", stringify!($($pattern)+), e),
            }
        }
    }

    #[test]
    fn env_var() {
        let expected = "logga";
        let key_str: &str = EnvVar::AwsAccessKeyId.into();
        env::set_var(key_str, &expected);
        let outcome = match s3_client::env_var(EnvVar::AwsAccessKeyId) {
            Ok(val) => val == expected,
            _ => false,
        };
        env::remove_var(key_str);
        assert!(outcome)
    }

    #[test]
    fn keychain_item_fail_to_read() {
        let user = whoami::username();
        assert_err!(
            s3_client::keychain_item(KeychainServices::AwsAccessKeyId, &user),
            Err(ClientError::KeychainReadFailed(_))
        )
    }

    #[test]
    fn keychain_item_correct_read() {
        let user = whoami::username();
        let expected = "topsecret";

        let _ = set_generic_password(
            KeychainServices::AwsAccessKeyId.into(),
            &user,
            &expected.as_bytes(),
        );
        let outcome =
            s3_client::keychain_item(KeychainServices::AwsAccessKeyId, &user).unwrap_or_default();

        let _ = delete_generic_password(KeychainServices::AwsAccessKeyId.into(), &user);
        assert_eq!(outcome, expected)
    }

    #[test]
    fn env_var_missing() {
        let outcome = s3_client::env_var(EnvVar::AwsAccessKeyId);
        assert_err!(
            outcome,
            Err(ClientError::CredentialNotSet(_, env::VarError::NotPresent))
        )
    }

    #[test]
    fn get_aws_credentials_env() {
        let expected_creds = CredentialsStore("abc".to_string(), "123".to_string());
        let access_key: &str = EnvVar::AwsAccessKeyId.into();
        let secret_key: &str = EnvVar::AwsSecretAccessKey.into();
        env::set_var(access_key, &expected_creds.0);
        env::set_var(secret_key, &expected_creds.1);

        let outcome = match s3_client::get_aws_credentials(&Configuration {
            s3: S3 {
                bucket: String::from("value"),
                region: String::from("value"),
                endpoint: String::from("value"),
                keychain_authentication: false,
            },
        }) {
            Ok(val) => val.0 == expected_creds.0 && val.1 == expected_creds.1,
            _ => false,
        };
        env::remove_var(access_key);
        env::remove_var(secret_key);
        assert!(outcome)
    }

    #[test]
    fn get_aws_credentials_keychain() {
        let expected_creds = CredentialsStore("abc".to_string(), "123".to_string());

        let user = whoami::username();
        let _ = set_generic_password(
            KeychainServices::AwsAccessKeyId.into(),
            &user,
            &expected_creds.0.as_bytes(),
        );

        let _ = set_generic_password(
            KeychainServices::AwsSecretAccessKey.into(),
            &user,
            &expected_creds.1.as_bytes(),
        );

        let outcome = match s3_client::get_aws_credentials(&Configuration {
            s3: S3 {
                bucket: String::from("value"),
                region: String::from("value"),
                endpoint: String::from("value"),
                keychain_authentication: true,
            },
        }) {
            Ok(val) => val.0 == expected_creds.0 && val.1 == expected_creds.1,
            _ => false,
        };
        let _ = delete_generic_password(KeychainServices::AwsAccessKeyId.into(), &user);
        let _ = delete_generic_password(KeychainServices::AwsSecretAccessKey.into(), &user);
        assert!(outcome)
    }

    #[tokio::test]
    async fn create_s3_client_missing_env() {
        let config = &Configuration {
            s3: S3 {
                bucket: String::from("dummy"),
                region: String::from("us-east-1"),
                endpoint: String::from("s3://endpoint"),
                keychain_authentication: false,
            },
        };
        let outcome = s3_client::create_s3_client(&config).await;

        assert_err!(
            outcome,
            Err(ClientError::CredentialNotSet(_, env::VarError::NotPresent))
        )
    }

    #[tokio::test]
    async fn create_s3_client_failing_keychain() {
        let config = &Configuration {
            s3: S3 {
                bucket: String::from("dummy"),
                region: String::from("us-east-1"),
                endpoint: String::from("s3://endpoint"),
                keychain_authentication: true,
            },
        };
        let outcome = s3_client::create_s3_client(&config).await;

        assert_err!(outcome, Err(ClientError::KeychainReadFailed(_)))
    }

    #[tokio::test]
    async fn create_s3_client() {
        let expected_creds = CredentialsStore("abc".to_string(), "123".to_string());

        let user = whoami::username();
        let _ = set_generic_password(
            KeychainServices::AwsAccessKeyId.into(),
            &user,
            &expected_creds.0.as_bytes(),
        );

        let _ = set_generic_password(
            KeychainServices::AwsSecretAccessKey.into(),
            &user,
            &expected_creds.1.as_bytes(),
        );

        let config = &Configuration {
            s3: S3 {
                bucket: String::from("dummy"),
                region: String::from("us-east-1"),
                endpoint: String::from("s3://endpoint"),
                keychain_authentication: true,
            },
        };
        let outcome = match s3_client::create_s3_client(&config).await {
            Ok(_) => true,
            _ => false,
        };

        let _ = delete_generic_password(KeychainServices::AwsAccessKeyId.into(), &user);
        let _ = delete_generic_password(KeychainServices::AwsSecretAccessKey.into(), &user);

        assert!(outcome);
    }
}
