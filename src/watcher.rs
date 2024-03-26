use log::info;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::Path;

pub fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => handle(&event),
            Err(error) => log::error!("Error: {error:?}"),
        }
    }

    Ok(())
}

fn handle(evt: &Event) {
    let fp = evt.paths[0].to_str().unwrap_or_default();
    if EventKind::is_create(&evt.kind)
        && Path::new(fp)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            == String::from("zip")
    {
        info!("found out little path {:?}", evt)
    }
}
