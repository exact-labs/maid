use crate::cli;
use crate::helpers;
use colored::Colorize;
use inquire::Select;
use just_macros::{crashln, errorln, string, ternary};
use optional_field::Field;
use std::{collections::HashMap, env};
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

pub fn hydrate(path: &String) {
    let values: cli::Maidfile = match toml::from_str(&helpers::read_maidfile(path)) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot read maidfile.");
        }
    };

    let mut table = HashMap::new();

    table.insert("os.platform", env::consts::OS);
    table.insert("os.arch", env::consts::ARCH);

    log::info!("{} os.platform: '{}'", helpers::add_icon(), env::consts::OS.yellow());
    log::info!("{} os.arch: '{}'", helpers::add_icon(), env::consts::ARCH.yellow());

    match env::current_dir() {
        Ok(path) => {
            table.insert("dir.current", helpers::path_to_str(&path));
            log::info!("{} dir.current: '{}'", helpers::add_icon(), helpers::path_to_str(&path).yellow());
        }
        Err(err) => {
            log::warn!("{err}");
            errorln!("Current directory could not be added as script variable.");
        }
    }

    match home::home_dir() {
        Some(path) => {
            table.insert("dir.home", helpers::path_to_str(&path));
            log::info!("{} dir.home: '{}'", helpers::add_icon(), helpers::path_to_str(&path).yellow());
        }
        None => {
            errorln!("Home directory could not be added as script variable.");
        }
    }

    for (key, value) in values.env.iter() {
        let value_formatted = ternary!(
            value.to_string().starts_with("\""),
            helpers::trim_start_end(helpers::string_to_static_str(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table))),
            helpers::string_to_static_str(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table))
        );

        env::set_var(key, value_formatted);
        log::info!("{} env.{key}: '{}'", helpers::add_icon(), value_formatted.yellow());
        table.insert(helpers::string_to_static_str(format!("env.{}", key.clone())), value_formatted);
    }

    let json = match serde_json::to_string(&values.tasks) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot read maidfile.");
        }
    };

    let hydrated_json = Template::new_with_placeholder(&json, "%{", "}").fill_with_hashmap(&table);

    println!("{hydrated_json}")
}

pub fn json(path: &String) {
    let values: cli::Maidfile = match toml::from_str(&helpers::read_maidfile(path)) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot read maidfile.");
        }
    };

    let json = match serde_json::to_string(&values) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot read maidfile.");
        }
    };

    println!("{json}")
}

pub fn list(path: &String, silent: bool, log_level: Option<log::Level>) {
    let values: cli::Maidfile = match toml::from_str(&helpers::read_maidfile(path)) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot read maidfile.");
        }
    };

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
