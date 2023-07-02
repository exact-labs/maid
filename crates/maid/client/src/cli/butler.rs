use crate::helpers;

use colored::Colorize;
use inquire::Text;
use macros_rs::string;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::{fs::File, io::Write, path::Path, time::Duration};

fn create_error(name: &str) {
    println!("An error happened when asking for {name}, try again later.");
    std::fs::remove_file("maidfile").unwrap();
    std::process::exit(1);
}

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

pub fn update() { println!("check and retrive updates") }

pub fn init() {
    let example_maidfile = "[tasks.example]\ninfo = \"this is a comment\"\nscript = \"echo 'hello world'\"";

    if !helpers::Exists::file(string!("maidfile")).unwrap() {
        println!("This utility will walk you through creating a maidfile.\n");

        let mut file = File::create("maidfile").unwrap();
        let current_dir = std::env::current_dir().unwrap();
        writeln!(&mut file, "[project]").unwrap();

        let name = Text::new("project name:").with_default(&current_dir.file_name().unwrap().to_str().unwrap().to_string()).prompt();
        let version = Text::new("version:").with_default("1.0.0").prompt();

        match name {
            Ok(name) => writeln!(&mut file, "name = \"{name}\"").unwrap(),
            Err(_) => create_error("project name"),
        }
        match version {
            Ok(version) => writeln!(&mut file, "version = \"{version}\"").unwrap(),
            Err(_) => create_error("version"),
        }

        writeln!(&mut file, "\n{example_maidfile}").unwrap();
        println!("{}", "\n✨ success, saved maidfile".yellow());
        if helpers::Exists::file(string!(".git")).unwrap() {
            println!("{}", "dont forget to add '.maid' to your .gitignore".white());
        }
    } else {
        println!("{}", "maidfile already exists, aborting".yellow())
    }
}

pub fn clean() {
    match std::fs::remove_dir_all(".maid/cache") {
        Ok(_) => println!("{}", "cleaned maid cache".green()),
        Err(_) => println!("{}", "maid cache does not exist, cannot remove".yellow()),
    };
}
