use crate::helpers;
use crate::structs::{Cache, CacheConfig, Task};
use crate::task;

use colored::Colorize;
use fs_extra::dir::get_size;
use global_placeholders::global;
use human_bytes::human_bytes;
use macros_rs::{crashln, fmtstr, string, ternary};
use std::{env, path::Path, time::Instant};

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

pub fn exec(task: &str, args: &Vec<String>, path: &String, silent: bool, is_dep: bool, log_level: Option<log::Level>) {
    log::info!("Starting maid {}", env!("CARGO_PKG_VERSION"));

    if task.is_empty() {
        tasks::list(path, silent, log_level)
    } else {
        let values = helpers::maidfile::merge(path);
        let cwd = &helpers::file::get_current_working_dir();

        if values.tasks.get(task).is_none() {
            crashln!("Maid could not find the task '{task}'. Does it exist?");
        }

        match &values.tasks[task].depends {
            Some(deps) => {
                let start = Instant::now();
                let ticks = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let template = fmtstr!("{{prefix:.white}} {{spinner:.yellow}}{{msg}} {}", "({elapsed})".bright_cyan());
                let pb = task::progress::init(ticks, template, 80);

                for (index, item) in deps.iter().enumerate() {
                    pb.set_prefix(format!("[{}/{}]", index + 1, deps.len()));
                    pb.set_message(fmtstr!("{} {item}", "running dependency".bright_yellow()));
                    exec(&item, args, path, true, true, log_level);
                }

                pb.suspend(|| {
                    println!(
                        "{} {} in {}\n",
                        helpers::string::check_icon(),
                        format!("finished {} dependencies", deps.len()).bright_green(),
                        format!("{:.2?}", start.elapsed()).yellow()
                    )
                });
            }
            None => {}
        };

        let cache = match &values.tasks[task].cache {
            Some(cache) => cache.clone(),
            None => Cache { path: string!(""), target: vec![] },
        };

        let task_path = match &values.tasks[task].path {
            Some(path) => ternary!(path == "", cwd, path),
            None => cwd,
        };

        if !cache.path.trim().is_empty() && !cache.target.is_empty() {
            if !helpers::Exists::folder(global!("maid.cache_dir", task)).unwrap() {
                std::fs::create_dir_all(global!("maid.cache_dir", task)).unwrap();
                log::debug!("created maid cache dir");
            }

            let hash = task::cache::create_hash(&cache.path);
            let config_path = format!(".maid/cache/{task}/{}.toml", task);

            if !helpers::Exists::file(config_path.clone()).unwrap() {
                match std::fs::write(
                    config_path.clone(),
                    toml::to_string(&CacheConfig {
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

            let json = match toml::from_str::<CacheConfig>(&contents) {
                Ok(contents) => contents,
                Err(err) => crashln!("Cannot read cache config: {err}"),
            };

            if json.hash == hash && !is_dep {
                println!("{}", "skipping task due to cached files".bright_magenta());

                for target in cache.target.clone() {
                    let cache_file = format!(".maid/cache/{task}/target/{}", Path::new(&target.clone()).file_name().unwrap().to_str().unwrap());

                    println!(
                        "{} ({})",
                        format!("copied target '{}' from cache", target.clone()).magenta(),
                        format!("{}", human_bytes(get_size(cache_file.clone()).unwrap() as f64).white())
                    );

                    match std::fs::copy(Path::new(&cache_file), target.clone()) {
                        Ok(_) => log::debug!("copied target file {}", target),
                        Err(err) => {
                            log::warn!("{err}");
                            crashln!("Cannot copy target file.");
                        }
                    };
                }

                std::process::exit(0);
            } else {
                match std::fs::write(
                    config_path.clone(),
                    toml::to_string(&CacheConfig {
                        target: cache.target,
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
            is_dep,
        });
    }
}

pub mod butler;
pub mod run;
pub mod tasks;