use log::{debug, error};
use logga_helper::configuration::Configuration;
use logga_helper::flags::Flags;
use logga_helper::s3_client;
use logga_helper::watcher::watch;
use std::process;

#[tokio::main]
async fn main() {
    env_logger::init();

    let flags = Flags::build();
    let config = Configuration::build(&flags);

    let client = match s3_client::create_s3_client(&config).await {
        Ok(client) => client,
        Err(err) => {
            error!("Couldn't create AWS client: {}", err);

            process::exit(1);
        }
    };

    debug!("Watching directory: {}", flags.watch_dir);
    if let Err(error) = watch(flags.watch_dir, &client, &config.s3.bucket).await {
        error!("Problem watching directory: {error:?}");
    }
}
