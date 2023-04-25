use colored::Colorize;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::{path::Path, time::Duration};

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

pub fn clean() {
    match std::fs::remove_dir_all(".maid/cache") {
        Ok(_) => println!("{}", "cleaned maid cache".green()),
        Err(_) => println!("{}", "maid cache does not exist, cannot remove".yellow()),
    };
}
