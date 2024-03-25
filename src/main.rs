use logga_helper::configuration::Configuration;
use logga_helper::flags::Flags;
use std::{env, process};

fn main() {
    let flags = match Flags::build(env::args()) {
        Ok(flags) => flags,
        Err(err_string) => {
            eprintln!("Problem parsing arguments: {err_string}");
            process::exit(1)
        }
    };

    let profile_config =
        Configuration::parse_configuration_profile(&flags.profile_path, &flags.bundle_id)
            .unwrap_or_default();

    println!("{}", profile_config.s3.endpoint);
    println!("{}", profile_config.s3.access_key);
    println!("{}", profile_config.s3.secret_key);

    let config = match Configuration::parse_config_yaml(&flags.config_path) {
        Ok(config) => config,
        Err(err_string) => {
            eprintln!("Problem parsing config yaml: {err_string}");
            process::exit(1)
        }
    };

    println!("{}", config.s3.access_key);
    println!("{}", config.s3.secret_key);
    println!("{}", config.s3.endpoint);
}
