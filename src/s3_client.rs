use crate::configuration::Configuration;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::Client;
use std::env::VarError;
use std::{env, fmt};

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

#[derive(Debug)]
pub enum ClientError<'a> {
    CredentialNotSet(&'a str, VarError),
}

impl fmt::Display for ClientError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::CredentialNotSet(key, err) => {
                write!(f, "{} {}", key, err)
            }
        }
    }
}

pub async fn create_s3_client(config: &Configuration) -> Result<Client, ClientError> {
    let region = env_var(EnvVar::AwsDefaultRegion).unwrap_or_else(|_| config.s3.region.clone());
    let access_key = env_var(EnvVar::AwsAccessKeyId)?;
    let secret_key = env_var(EnvVar::AwsSecretAccessKey)?;
    let region_provider = Region::new(region);

    let credentials = Credentials::new(access_key, secret_key, None, None, "cloudflare");
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

fn env_var(key: EnvVar) -> Result<String, ClientError<'static>> {
    let key_str: &str = key.into();
    env::var(key_str).map_err(|e| ClientError::CredentialNotSet(key_str, e))
}
