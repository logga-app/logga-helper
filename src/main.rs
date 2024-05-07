mod configuration;
mod flags;
mod forwarder;
mod uploader;

use crate::configuration::Configuration;
use crate::flags::Flags;
use forwarder::network::Transmitter;
use forwarder::tail::Tail;
use log::{debug, error};
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use uploader::s3_client;
use uploader::watcher;

#[tokio::main]
async fn main() {
    env_logger::init();
    let signals = Signals::new(&[SIGINT, SIGTERM]);
    let flags = Flags::build();
    let config = Configuration::build(&flags);

    let client = match s3_client::create_s3_client(&config).await {
        Ok(client) => client,
        Err(err) => {
            error!("Couldn't create AWS client: {}", err);

            process::exit(1);
        }
    };

    // -------------------------------------------------

    let tailer = match Tail::new(flags.access_log_path) {
        Ok(t) => t,
        Err(err) => {
            error!("creating tailer: {}", err);
            std::process::exit(1);
        }
    };
    let tailer = Arc::new(Mutex::new(tailer));
    let signal_tailer = tailer.clone();
    // Start tailing the access log file
    let handle = thread::spawn(move || run_log_tailer(tailer.clone()));

    // -------------------------------------------------

    // Have tailer save current checkpoint on Signals & exit gracefully
    thread::spawn(move || {
        if let Ok(mut signals) = signals {
            for _ in signals.forever() {
                signal_tailer.lock().unwrap().save_checkpoint();
                debug!("stopping gracefully...");
                process::exit(0);
            }
        }
    });

    // -------------------------------------------------

    // Start watching the specified directory for rotated archives
    debug!("Watching directory: {}", flags.watch_dir);
    if let Err(error) = watcher::watch(&flags.watch_dir, &client, &config.s3.bucket).await {
        error!("Problem watching directory: {error:?}");
    }

    handle.join().unwrap();
}

fn run_log_tailer(tailer: Arc<Mutex<Tail>>) {
    let tr = Transmitter::new("TODO_URL");

    loop {
        match tailer.lock().unwrap().next_then(|data| {
            // tr.transmit_data_chunk(data.into());
            print!("{}", std::str::from_utf8(&data).unwrap());
        }) {
            Err(err) => error!("tail error: {}", err),
            _ => {}
        }

        sleep(Duration::from_millis(100));
    }
}
