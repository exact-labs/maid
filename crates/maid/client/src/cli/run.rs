use crate::cli;
use crate::helpers;
use crate::shell::IntoArgs;
use crate::structs::{Cache, Runner};
use crate::table;

use colored::Colorize;
use fs_extra::dir::get_size;
use human_bytes::human_bytes;
use macros_rs::{crashln, string};
use serde_json::json;
use std::env;
use std::io::Error;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::Instant;
use text_placeholder::Template;

fn run_script(runner: Runner) {
    let mut cmd: Child;
    let start = Instant::now();
    let mut status_array: Vec<Result<ExitStatus, Error>> = vec![];

    for string in runner.script.clone() {
        let start = Instant::now();
        let table = table::create(runner.maidfile.clone(), runner.args, runner.project.clone());
        let script = Template::new_with_placeholder(string, "%{", "}").fill_with_hashmap(&table);
        let (name, args) = match script.try_into_args() {
            Ok(result) => {
                let mut args = result.clone();

                args.remove(0);
                (result[0].clone(), args)
            }
            Err(err) => {
                log::warn!("{err}");
                crashln!("Script could not be parsed into args");
            }
        };

        log::debug!("Original Script: {}", string);
        log::debug!("Parsed Script: {}", script);
        log::trace!("{}", json!({"name": name, "args": args}));
        log::info!("Execute Command: '{name} {}'", args.join(" "));

        let working_dir = runner.project.join(&Path::new(runner.path));
        match env::set_current_dir(&working_dir) {
            Ok(_) => {
                log::info!("Working directory: {:?}", &working_dir);
            }
            Err(err) => {
                crashln!("Failed to set working directory {:?}\nError: {:#?}", &working_dir, err);
            }
        };

        if runner.is_dep {
            cmd = match Command::new(&name).stdout(Stdio::null()).stderr(Stdio::null()).stdin(Stdio::null()).args(args.clone()).spawn() {
                Ok(output) => output,
                Err(err) => {
                    log::warn!("{err}");
                    crashln!("Cannot start command {name}.");
                }
            };
        } else {
            cmd = match Command::new(&name).args(args.clone()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).stdin(Stdio::inherit()).spawn() {
                Ok(output) => output,
                Err(err) => {
                    log::warn!("{err}");
                    crashln!("Cannot start command {name}.");
                }
            };
        }

        let status = cmd.wait();
        let exit_code = helpers::status::code(&status);

        status_array.push(status);
        log::debug!("Finished cmd: '{name} {}' with exit code: {:?} in {:.2?}", args.join(" "), exit_code, start.elapsed());
    }

    let status = match status_array.last() {
        Some(status) => status,
        None => crashln!("Failed to fetch final status code."),
    };

    let cache = match &runner.maidfile.tasks[runner.name].cache {
        Some(cache) => cache.clone(),
        None => Cache { path: string!(""), target: vec![] },
    };

    let exit_code = helpers::status::code(status);
    let success = helpers::status::success(&status);

    if !runner.silent {
        if success {
            println!("\n{} {}", helpers::string::check_icon(), "finished task successfully".bright_green());
            if !cache.path.trim().is_empty() && !cache.target.is_empty() {
                for target in cache.target {
                    let cache_file = format!(".maid/cache/{}/target/{}", runner.name, Path::new(&target).file_name().unwrap().to_str().unwrap());
                    match std::fs::copy(Path::new(&target), cache_file.clone()) {
                        Ok(_) => {
                            println!(
                                "{} ({})",
                                format!("saved target '{}' to cache", target).bright_magenta(),
                                format!("{}", human_bytes(get_size(cache_file.clone()).unwrap() as f64).white())
                            );
                            log::debug!("saved target file {}", target)
                        }
                        Err(err) => {
                            log::warn!("{err}");
                            log::debug!("path: {}", target);
                            crashln!("Cannot save target file.");
                        }
                    };
                }
            }
            println!("{} took {}", runner.name.white(), format!("{:.2?}", start.elapsed()).yellow());
        } else {
            println!("\n{} {} {}", helpers::string::cross_icon(), "exited with status code".bright_red(), format!("{}", exit_code).red());
            println!("{} took {}", runner.name.white(), format!("{:.2?}", start.elapsed()).yellow());
        }
    } else {
        if success {
            if !cache.path.trim().is_empty() && !cache.target.is_empty() {
                for target in cache.target {
                    let cache_file = format!(".maid/cache/{}/target/{}", runner.name, Path::new(&target).file_name().unwrap().to_str().unwrap());
                    match std::fs::copy(Path::new(&target), cache_file.clone()) {
                        Ok(_) => println!(
                            "{} {}{}{}",
                            helpers::string::add_icon(),
                            format!("{}", target).bright_green(),
                            helpers::string::seperator(),
                            format!("{}", human_bytes(get_size(cache_file.clone()).unwrap() as f64).bright_cyan())
                        ),
                        Err(err) => {
                            log::error!("{err}");
                            log::debug!("path: {}", target);
                        }
                    };
                }
            }
        }
    }
}

pub fn task(task: cli::Task) {
    let mut script: Vec<&str> = vec![];

    if task.script.is_str() {
        match task.script.as_str() {
            Some(cmd) => script.push(cmd),
            None => crashln!("Unable to parse maidfile. Missing string value."),
        };
    } else if task.script.is_array() {
        match IntoIterator::into_iter(match task.script.as_array() {
            Some(iter) => iter,
            None => crashln!("Unable to parse maidfile. Missing array value."),
        }) {
            mut iter => loop {
                match Iterator::next(&mut iter) {
                    Some(val) => match val.as_str() {
                        Some(cmd) => script.push(cmd),
                        None => crashln!("Unable to parse maidfile. Missing string value."),
                    },
                    None => break,
                };
            },
        }
    } else {
        helpers::status::error(task.script.type_str())
    }

    run_script(Runner {
        name: &task.name,
        path: &task.path,
        args: &task.args,
        silent: task.silent,
        is_dep: task.is_dep,
        project: &task.project,
        maidfile: &task.maidfile,
        script,
    });
}
