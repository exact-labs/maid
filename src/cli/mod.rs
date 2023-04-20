use crate::helpers;
use colored::Colorize;
use just_macros::{crashln, ternary};
use optional_field::{serde_optional_fields, Field};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, env, time::Instant};
use toml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct Maidfile {
    pub env: BTreeMap<String, Value>,
    pub tasks: BTreeMap<String, Tasks>,
}

#[serde_optional_fields]
#[derive(Debug, Deserialize, Serialize)]
pub struct Tasks {
    pub script: Value,
    pub path: Field<String>,
    pub info: Field<String>,
    pub hide: Field<bool>,
}

pub fn exec(task: &String, args: &Vec<String>, path: &String, silent: bool, log_level: Option<log::Level>) {
    log::info!("Starting maid {}", env!("CARGO_PKG_VERSION"));
    if task == "" {
        tasks::list(path, silent, log_level)
    } else {
        let start = Instant::now();
        let cwd = &helpers::get_current_working_dir();

        let values: Maidfile = match toml::from_str(&helpers::read_maidfile(path)) {
            Ok(contents) => contents,
            Err(err) => {
                log::warn!("{err}");
                crashln!("Cannot read maidfile.");
            }
        };

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
