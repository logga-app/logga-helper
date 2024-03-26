use log::{error, info};
use logga_helper::configuration::Configuration;
use logga_helper::flags::Flags;
use logga_helper::watcher::watch;
use std::{env, process};

fn main() {
    env_logger::init();

    let flags = match Flags::build(env::args()) {
        Ok(flags) => flags,
        Err(err_string) => {
            error!("Problem parsing arguments: {err_string}");
            process::exit(1)
        }
    };

    let config = Configuration::build(&flags.config_path, &flags.profile_path, &flags.bundle_id);

    info!("{}", config.s3.access_key);
    info!("{}", config.s3.secret_key);
    info!("{}", config.s3.endpoint);

    if let Err(error) = watch(".") {
        error!("Problem watching directory: {error:?}");
    }
}
