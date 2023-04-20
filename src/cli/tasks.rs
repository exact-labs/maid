use crate::cli;
use crate::helpers;
use colored::Colorize;
use inquire::Select;
use just_macros::{crashln, errorln, ternary};
use optional_field::Field;
use std::{collections::HashMap, env, str::from_utf8};
use text_placeholder::Template;

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

    let mut options = values
        .tasks
        .iter()
        .map(|(key, task)| {
            let task_info = match &task.info {
                Field::Present(Some(info)) => format!(" ({})", info).white(),
                Field::Present(None) => format!(" {}", "(no description)").bright_red(),
                Field::Missing => format!(" {}", "(no description)").bright_red(),
            };

            let verbose_task = ternary!(log_level.unwrap() == log::Level::Warn, format!(" {}", task.script).bright_blue(), String::from("").bright_blue());
            let formatted_task = format!("{}{}{}", format!("{key}").bright_yellow(), task_info, verbose_task);
            ternary!(key.starts_with("_"), String::from("removed_item"), formatted_task)
        })
        .collect::<Vec<_>>();

    options.retain(|x| x != "removed_item");
    match Select::new("Select a task to run:", options).prompt() {
        Ok(task) => {
            let key = &strip_ansi_escapes::strip(&task.split(" ").collect::<Vec<&str>>()[0]).unwrap();
            let name = from_utf8(key).unwrap();

            log::debug!("Starting {name}");
            cli::exec(&String::from(name), &vec![String::from("")], &path, silent, log_level);
        }
        Err(_) => println!("{}", "Aborting...".white()),
    }
}
