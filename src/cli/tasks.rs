use crate::cli;
use crate::helpers;
use colored::Colorize;
use inquire::Select;
use just_macros::{crashln, ternary};
use std::str::from_utf8;

pub fn list(path: &String, silent: bool, log_level: Option<log::Level>) {
    let values: cli::Maidfile = match toml::from_str(&helpers::read_maidfile(path)) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot read maidfile.");
        }
    };

    let mut options = values
        .tasks
        .iter()
        .map(|(key, task)| {
            let verbose_task = ternary!(log_level.unwrap() == log::Level::Warn, format!(" {}", task.script).bright_blue(), String::from("").bright_blue());
            let formatted_task = format!("{}: {}{}", format!("{key}").bright_yellow(), format!("{}", task.info).white(), verbose_task);
            ternary!(key.starts_with("_"), String::from("removed_item"), formatted_task)
        })
        .collect::<Vec<_>>();

    options.retain(|x| x != "removed_item");
    match Select::new("Select a task to run:", options).prompt() {
        Ok(task) => {
            let key = &strip_ansi_escapes::strip(&task.split(":").collect::<Vec<&str>>()[0]).unwrap();
            let name = from_utf8(key).unwrap();

            log::debug!("Starting {name}");
            cli::exec(&String::from(name), &vec![String::from("")], &path, silent, log_level);
        }
        Err(_) => println!("{}", "Aborting...".white()),
    }
}
