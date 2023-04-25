use std::{path::Path, time::Duration};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;

pub fn watch(path: &Path) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx).unwrap();

    debouncer.watcher().watch(path, RecursiveMode::Recursive).unwrap();
    for events in rx {
        if let Ok(event) = events {
            println!("{:?}", event);
        }
    }
}
