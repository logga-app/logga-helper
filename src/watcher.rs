use aws_sdk_s3::Client;
use log::{error, info, warn};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
enum ExtensionKey {
    Zip,
}

impl fmt::Display for ExtensionKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
pub async fn watch<P: AsRef<Path>>(
    path: P,
    client: &Client,
    bucket: &String,
) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => handle(&event, &client, &bucket).await,
            Err(error) => log::error!("Error: {error:?}"),
        }
    }

    Ok(())
}

async fn handle(evt: &Event, client: &Client, bucket: &String) {
    if evt.kind.is_create() || evt.kind.is_remove() {
        match evt.paths.first() {
            Some(p) => {
                let path_str = p.to_str().unwrap_or_default();
                let file_path = Path::new(path_str);

                if &file_path
                    .extension()
                    .and_then(OsStr::to_str)
                    .unwrap_or_default()
                    == &ExtensionKey::Zip.to_string().to_lowercase()
                {
                    info!("uploading file: {:?}", &file_path);
                    let request = client
                        .put_object()
                        .bucket(bucket)
                        .key(
                            file_path
                                .file_name()
                                .and_then(OsStr::to_str)
                                .unwrap_or_default(),
                        )
                        .send()
                        .await;

                    match request {
                        Ok(_) => info!("{:?} backed up successfully", &file_path),
                        Err(err) => error!("Problem uploading {:?}: {:?}", &file_path, err),
                    }
                }
            }
            None => warn!("{:?} paths was empty", evt.kind),
        }
    }
}
