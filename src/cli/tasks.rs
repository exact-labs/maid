use crate::cli;
use crate::helpers;
use colored::Colorize;
use inquire::Select;
use just_macros::{string, ternary};
use optional_field::Field;
use text_placeholder::Template;

#[derive(Debug)]
struct Task {
    name: String,
    formatted: String,
    hidden: bool,
}

impl std::fmt::Display for Task {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.formatted, f)
    }
}

pub fn json(path: &String, args: &Vec<String>, hydrate: &bool) {
    let values = helpers::parse_maidfile(path);
    let json = helpers::toml_to_json(values.clone());
    let table = cli::create_table(values.clone(), args);
    let hydrated_json = Template::new_with_placeholder(&json, "%{", "}").fill_with_hashmap(&table);

    println!("{}", ternary!(hydrate.clone(), hydrated_json, json))
}

pub fn list(path: &String, silent: bool, log_level: Option<log::Level>) {
    let values = helpers::parse_maidfile(path);
    let mut options: Vec<_> = values
        .tasks
        .iter()
        .map(|(key, task)| {
            let info = match &task.info {
                Field::Present(Some(info)) => format!("({info})").white(),
                Field::Present(None) => string!("(no description)").bright_red(),
                Field::Missing => string!("(no description)").bright_red(),
            };

            let verbose = match log_level.unwrap() {
                log::Level::Error => string!(),
                _ => string!(task.script),
            };

            let hidden = match key.starts_with("_") {
                true => true,
                false => match task.hide {
                    Field::Present(Some(val)) => val,
                    Field::Present(None) => false,
                    Field::Missing => false,
                },
            };

            return Task {
                name: key.clone(),
                formatted: format!("{} {} {}", format!("{key}").bright_yellow(), info, verbose.bright_blue()),
                hidden: hidden.clone(),
            };
        })
        .collect();

    options.retain(|key| key.hidden == false);
    match Select::new("Select a task to run:", options).prompt() {
        Ok(task) => {
            log::debug!("Starting {}", task.name);
            cli::exec(&String::from(task.name), &vec![String::from("")], &path, silent, log_level);
        }
        Err(_) => println!("{}", "Aborting...".white()),
    }
}
