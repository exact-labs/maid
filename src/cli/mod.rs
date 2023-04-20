use crate::helpers;
use colored::Colorize;
use macros_rs::{crashln, errorln, ternary};
use optional_field::{serde_optional_fields, Field};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, collections::HashMap, env, time::Instant};
use text_placeholder::Template;
use toml::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Maidfile {
    pub env: BTreeMap<String, Value>,
    pub tasks: BTreeMap<String, Tasks>,
}

#[serde_optional_fields]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tasks {
    pub script: Value,
    pub path: Field<String>,
    pub info: Field<String>,
    pub hide: Field<bool>,
    pub depends: Field<Value>,
}

pub fn create_table(values: Maidfile, args: &Vec<String>) -> HashMap<&str, &str> {
    let mut table = HashMap::new();

    table.insert("os.platform", env::consts::OS);
    table.insert("os.arch", env::consts::ARCH);

    log::info!("{} os.platform: '{}'", helpers::string::add_icon(), env::consts::OS.yellow());
    log::info!("{} os.arch: '{}'", helpers::string::add_icon(), env::consts::ARCH.yellow());

    match env::current_dir() {
        Ok(path) => {
            table.insert("dir.current", helpers::string::path_to_str(&path));
            log::info!("{} dir.current: '{}'", helpers::string::add_icon(), helpers::string::path_to_str(&path).yellow());
        }
        Err(err) => {
            log::warn!("{err}");
            errorln!("Current directory could not be added as script variable.");
        }
    }

    match home::home_dir() {
        Some(path) => {
            table.insert("dir.home", helpers::string::path_to_str(&path));
            log::info!("{} dir.home: '{}'", helpers::string::add_icon(), helpers::string::path_to_str(&path).yellow());
        }
        None => {
            errorln!("Home directory could not be added as script variable.");
        }
    }

    for (pos, arg) in args.iter().enumerate() {
        log::info!("{} arg.{pos}: '{}'", helpers::string::add_icon(), arg.yellow());
        table.insert(helpers::string::to_static_str(format!("arg.{pos}")), arg);
    }

    for (key, value) in values.env.iter() {
        let value_formatted = ternary!(
            value.to_string().starts_with("\""),
            helpers::string::trim_start_end(helpers::string::to_static_str(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table))),
            helpers::string::to_static_str(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table))
        );

        env::set_var(key, value_formatted);
        log::info!("{} env.{key}: '{}'", helpers::string::add_icon(), value_formatted.yellow());
        table.insert(helpers::string::to_static_str(format!("env.{}", key.clone())), value_formatted);
    }

    return table;
}

pub fn exec(task: &String, args: &Vec<String>, path: &String, silent: bool, log_level: Option<log::Level>) {
    log::info!("Starting maid {}", env!("CARGO_PKG_VERSION"));
    if task == "" {
        tasks::list(path, silent, log_level)
    } else {
        let start = Instant::now();
        let cwd = &helpers::file::get_current_working_dir();
        let values = &helpers::file::read_maidfile(path);

        if values.tasks.get(task).is_none() {
            crashln!("Maid could not find the task '{task}'. Does it exist?");
        }

        let task_path = match &values.tasks[task].path {
            Field::Present(Some(path)) => ternary!(path == "", cwd, path),
            Field::Present(None) => cwd,
            Field::Missing => cwd,
        };

        log::debug!("Task path: {}", task_path);
        log::debug!("Working dir: {}", cwd);
        log::debug!("Started task: {}", task);

        if !silent {
            ternary!(
                task_path == cwd,
                println!("{} {}", "»".white(), &values.tasks[task].script),
                println!("{} {} {}", format!("({task_path})").bright_cyan(), "»".white(), &values.tasks[task].script)
            )
        }
        run::task(&values, &values.tasks[task].script, task_path, args);
        if !silent {
            println!("\n{} {}", "✔".green(), "finished task successfully".bright_green());
            println!("{} took {}", task.white(), format!("{:.2?}", start.elapsed()).yellow());
        }
    }
}

pub mod run;
pub mod tasks;
