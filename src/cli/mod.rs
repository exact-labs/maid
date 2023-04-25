use crate::helpers;
use crate::structs::{Cache, CacheConfig, Task};
use crate::task;

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
        let cwd = &helpers::file::get_current_working_dir();

        if values.tasks.get(task).is_none() {
            crashln!("Maid could not find the task '{task}'. Does it exist?");
        }

        let cache = match &values.tasks[task].cache {
            Some(cache) => cache.clone(),
            None => Cache {
                path: string!(""),
                target: string!(""),
            },
        };

        let task_path = match &values.tasks[task].path {
            Some(path) => ternary!(path == "", cwd, path),
            None => cwd,
        };

        if !cache.path.trim().is_empty() && !cache.target.trim().is_empty() {
            if !helpers::Exists::folder(format!(".maid/cache/{task}/target")).unwrap() {
                std::fs::create_dir_all(format!(".maid/cache/{task}/target")).unwrap();
                log::debug!("created maid cache dir");
            }

            let hash = task::cache::create_hash(&cache.path);
            let config_path = format!(".maid/cache/{task}/config.json");

            if !helpers::Exists::file(config_path.clone()).unwrap() {
                match std::fs::write(
                    config_path.clone(),
                    serde_json::to_string(&CacheConfig {
                        target: cache.target.clone(),
                        hash: string!(""),
                    })
                    .unwrap(),
                ) {
                    Ok(_) => log::debug!("created {task} cache config"),
                    Err(err) => crashln!("error {err} creating cache config"),
                };
            }

            let contents = match std::fs::read_to_string(config_path.clone()) {
                Ok(content) => content,
                Err(err) => crashln!("Cannot read cache config: {err}"),
            };

            let json = match serde_json::from_str::<CacheConfig>(&contents) {
                Ok(contents) => contents,
                Err(err) => crashln!("Cannot read cache config: {err}"),
            };

            if json.hash == hash {
                println!("{}", "skipping task due to cached files".bright_magenta());
                println!("{}", format!("copied target '{}' from cache", cache.target.clone()).magenta());

                match std::fs::copy(format!(".maid/cache/{task}/target/{}", cache.target.clone()), format!("{}", cache.target.clone())) {
                    Ok(_) => log::debug!("copied target file {}", cache.target.clone()),
                    Err(err) => {
                        log::warn!("{err}");
                        crashln!("Cannot copy target file.");
                    }
                };

                std::process::exit(0);
            } else {
                match std::fs::write(
                    config_path.clone(),
                    serde_json::to_string(&CacheConfig {
                        target: cache.target.clone(),
                        hash: hash.clone(),
                    })
                    .unwrap(),
                ) {
                    Ok(_) => log::debug!("added hash for {task} -> {hash}"),
                    Err(err) => crashln!("error {err} creating cache config"),
                };
            }
        };

        log::debug!("Task path: {task_path}");
        log::debug!("Working dir: {cwd}");
        log::debug!("Started task: {task}");

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

pub mod butler;
pub mod run;
pub mod tasks;
