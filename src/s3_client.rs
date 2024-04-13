use crate::configuration::Configuration;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::{BehaviorVersion, Credentials};
use aws_sdk_s3::Client;
use std::env;

pub async fn create_s3_client(config: &Configuration) -> Result<Client, &'static str> {
    let access_key = env::var("AWS_ACCESS_KEY_ID").unwrap_or_default();
    let secret_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default();
    if access_key.is_empty() || secret_key.is_empty() {
        return Err("AWS credentials not set");
    }

    let credentials = Credentials::new(access_key, secret_key, None, None, "cloudflare");
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
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
