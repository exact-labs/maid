use crate::helpers;
use crate::parse;
use crate::structs::Task;

use colored::Colorize;
use macros_rs::{crashln, string, ternary};
use std::env;

pub fn info(path: &String) {
    let values = helpers::maidfile::merge(path);

    let name = match &values.project {
        Some(project) => match project.name.clone() {
            Some(name) => name,
            None => string!("none"),
        },
        None => string!("none"),
    };

    let version = match &values.project {
        Some(project) => match project.version.clone() {
            Some(version) => version,
            None => string!("none"),
        },
        None => string!("none"),
    };

    println!(
        "{}\n{}\n{}",
        "Project Info".green().bold(),
        format!(" {}: {}", "- Name".white(), name.bright_yellow()),
        format!(" {}: {}", "- Version".white(), version.bright_yellow())
    );
}

pub fn exec(task: &str, args: &Vec<String>, path: &String, silent: bool, log_level: Option<log::Level>) {
    log::info!("Starting maid {}", env!("CARGO_PKG_VERSION"));

    if task.is_empty() {
        tasks::list(path, silent, log_level)
    } else {
        let values = helpers::maidfile::merge(path);
        let cwd = &parse::file::get_current_working_dir();

        if values.tasks.get(task).is_none() {
            crashln!("Maid could not find the task '{task}'. Does it exist?");
        }

        let task_path = match &values.tasks[task].path {
            Some(path) => ternary!(path == "", cwd, path),
            None => cwd,
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

        run::task(Task {
            maidfile: values.clone(),
            name: string!(task),
            script: values.tasks[task].script.clone(),
            path: task_path.clone(),
            args: args.clone(),
            silent,
        });
    }
}

pub mod run;
pub mod tasks;
