use crate::helpers;
use colored::Colorize;
use macros_rs::{crashln, errorln, string, ternary};
use merge_struct::merge;
use optional_field::Field;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeMap, collections::HashMap, env};
use text_placeholder::Template;
use toml::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Maidfile {
    pub import: Option<Vec<String>>,
    pub env: Field<BTreeMap<String, Value>>,
    pub project: Field<Project>,
    pub tasks: BTreeMap<String, Tasks>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: Field<String>,
    pub version: Field<String>,
    pub server: Field<Server>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Server {
    pub address: Address,
    pub token: Field<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tasks {
    pub script: Value,
    pub retry: Option<i32>,
    pub hide: Field<bool>,
    pub cache: Field<bool>,
    pub path: Field<String>,
    pub info: Field<String>,
    pub target: Field<Value>,
    pub remote: Field<Remote>,
    pub depends: Field<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Address {
    pub ip: Field<String>,
    pub port: Field<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Remote {
    pub push: Field<Value>,
    pub pull: Field<Value>,
    pub worker: Field<String>,
    pub dependencies: Field<Value>,
}

pub fn info(path: &String) {
    let values = read_maidfile_merge(path);

    let name = match &values.project {
        Field::Present(Some(project)) => project.name.clone().unwrap(),
        Field::Present(None) => string!("none"),
        Field::Missing => string!("none"),
    };

    let version = match &values.project {
        Field::Present(Some(project)) => project.version.clone().unwrap(),
        Field::Present(None) => string!("none"),
        Field::Missing => string!("none"),
    };

    println!(
        "{}\n{}\n{}",
        "Project Info".green().bold(),
        format!(" {}: {}", "- Name".white(), name.bright_yellow()),
        format!(" {}: {}", "- Version".white(), version.bright_yellow())
    );
}

pub fn read_maidfile_merge(path: &String) -> Maidfile {
    let mut values = helpers::file::read_maidfile(path);
    let imported_values = import::tasks(values.import.clone());

    for import in imported_values.iter() {
        values = match merge(&values, &import) {
            Ok(merge) => merge,
            Err(err) => {
                log::warn!("{err}");
                crashln!("Unable to import tasks.");
            }
        };
    }

    return values;
}

pub fn create_table(values: Maidfile, args: &Vec<String>) -> HashMap<&str, &str> {
    let mut table = HashMap::new();
    let empty_env: BTreeMap<String, Value> = BTreeMap::new();

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

    let user_env = match &values.env {
        Field::Present(Some(env)) => env.iter(),
        Field::Present(None) => empty_env.iter(),
        Field::Missing => empty_env.iter(),
    };

    for (key, value) in user_env {
        let value_formatted = ternary!(
            value.to_string().starts_with("\""),
            helpers::string::trim_start_end(helpers::string::to_static_str(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table))),
            helpers::string::to_static_str(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table))
        );

        env::set_var(key, value_formatted);
        log::info!("{} env.{key}: '{}'", helpers::string::add_icon(), value_formatted.yellow());
        table.insert(helpers::string::to_static_str(format!("env.{}", key.clone())), value_formatted);
    }

    log::trace!("{}", json!({ "env": table }));

    return table;
}

pub fn exec(task: &String, args: &Vec<String>, path: &String, silent: bool, log_level: Option<log::Level>) {
    log::info!("Starting maid {}", env!("CARGO_PKG_VERSION"));
    if task == "" {
        tasks::list(path, silent, log_level)
    } else {
        let values = read_maidfile_merge(path);
        let cwd = &helpers::file::get_current_working_dir();

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
                println!("{} {}", helpers::string::arrow_icon(), &values.tasks[task].script),
                println!("{} {} {}", format!("({task_path})").bright_cyan(), helpers::string::arrow_icon(), &values.tasks[task].script)
            )
        }
        run::task(&values, task, &values.tasks[task].script, task_path, args, silent);
    }
}

pub mod import;
pub mod run;
pub mod tasks;
