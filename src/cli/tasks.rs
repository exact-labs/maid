use crate::cli;
use crate::helpers;
use colored::Colorize;
use inquire::Select;
use just_macros::crashln;
use std::str::from_utf8;

pub fn list(path: &String, silent: bool) {
    let values: cli::Maidfile = match toml::from_str(&helpers::read_maidfile(path)) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot read maidfile.");
        }
    };

    let options = values
        .tasks
        .iter()
        .map(|(key, val)| format!("{}{} {}", format!("({key})").bright_magenta(), ":".white(), format!("{}", val.script).bright_yellow()))
        .collect::<Vec<_>>();

    match Select::new("Select a task to run:", options).prompt() {
        Ok(task) => {
            let key = &strip_ansi_escapes::strip(&task.split(":").collect::<Vec<&str>>()[0]).unwrap();
            let name = helpers::trim_start_end(from_utf8(key).unwrap());

            log::debug!("Starting {name}");
            cli::exec(&String::from(name), &vec![String::from("")], &path, silent);
        }
        Err(_) => println!("{}", "Aborting...".white()),
    }
}
