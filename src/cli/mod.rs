use crate::helpers;
use colored::Colorize;
use just_macros::{crashln, ternary};
use serde_derive::Deserialize;
use std::{collections::BTreeMap, env, time::Instant};
use toml::Value;

#[derive(Debug, Deserialize)]
pub struct Maidfile {
    pub env: BTreeMap<String, Value>,
    pub tasks: BTreeMap<String, Tasks>,
}

#[derive(Debug, Deserialize)]
pub struct Tasks {
    pub script: Value,
    pub path: String,
    pub info: String,
}

pub fn exec(task: &String, args: &Vec<String>, path: &String, silent: bool, log_level: Option<log::Level>) {
    log::info!("starting maid {}", env!("CARGO_PKG_VERSION"));
    if task == "" {
        tasks::list(path, silent, log_level)
    } else {
        let start = Instant::now();
        let values: Maidfile = match toml::from_str(&helpers::read_maidfile(path)) {
            Ok(contents) => contents,
            Err(err) => {
                log::warn!("{err}");
                crashln!("Cannot read maidfile.");
            }
        };

        let cwd = &String::from(env::current_dir().unwrap().to_string_lossy());
        log::info!("Working dir: {}", cwd);

        let task_path = ternary!(&values.tasks[task].path != "", &values.tasks[task].path, cwd);
        log::info!("Task: {}", task);

        if !silent {
            let formatted_path = format!("({})", task_path.split('/').last().unwrap());
            ternary!(
                task_path == cwd,
                println!("{} {}", "»".white(), &values.tasks[task].script),
                println!("{} {} {}", formatted_path.bright_cyan(), "»".white(), &values.tasks[task].script)
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
