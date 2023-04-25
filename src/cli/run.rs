use crate::cli;
use crate::helpers;
use crate::shell::IntoArgs;
use crate::structs::{Cache, Runner};
use crate::table;

use colored::Colorize;
use macros_rs::{crashln, inc, string};
use serde_json::json;
use std::io::Error;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Instant;
use text_placeholder::Template;

fn run_script(runner: Runner, mut retry_times: i32) {
    let start = Instant::now();
    let mut status_array: Vec<Result<ExitStatus, Error>> = vec![];

    let retry = match runner.maidfile.tasks[runner.name].retry {
        Some(retry) => retry,
        None => 0,
    };

    for string in runner.script.clone() {
        let start = Instant::now();
        let table = table::create(runner.maidfile.clone(), runner.args);
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

        let mut cmd = match Command::new(&name)
            .args(args.clone())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .current_dir(runner.path)
            .spawn()
        {
            Ok(output) => output, // add silent no output mode when in deps unless verbose
            Err(err) => {
                log::warn!("{err}");
                crashln!("Cannot start command {name}.");
            }
        };

        let status = cmd.wait();
        let exit_code = helpers::status::code(&status);
        let success = helpers::status::success(&status);

        if !success && retry >= 1 {
            print!("\n");

            if retry > retry_times {
                inc!(retry_times);
                println!("{} {} {}", helpers::string::warn_icon(), "retry attempt".bright_yellow(), format!("{}", retry_times).yellow());

                run_script(
                    Runner {
                        name: runner.name,
                        path: runner.path,
                        args: runner.args,
                        silent: runner.silent,
                        maidfile: runner.maidfile,
                        script: runner.script.clone(),
                    },
                    retry_times,
                );
            }
        }

        status_array.push(status);
        log::debug!("Finished cmd: '{name} {}' with exit code: {:?} in {:.2?}", args.join(" "), exit_code, start.elapsed());
    }

    let status = match status_array.last() {
        Some(status) => status,
        None => crashln!("Failed to fetch final status code."),
    };

    let cache = match &runner.maidfile.tasks[runner.name].cache {
        Some(cache) => cache.clone(),
        None => Cache {
            path: string!(""),
            target: string!(""),
        },
    };

    let exit_code = helpers::status::code(status);
    let success = helpers::status::success(&status);

    if !success && retry > retry_times && retry >= 1 {
        crashln!(
            "{} {} {}",
            helpers::string::cross_icon(),
            format!("max of {} retries reached\n - exited with status code", pretty_number::value(retry as u16)).bright_red(),
            format!("{}", exit_code).red()
        );
    }

    if !runner.silent && retry_times < 1 {
        if success {
            println!("\n{} {}", helpers::string::check_icon(), "finished task successfully".bright_green());
            if !cache.path.trim().is_empty() && !cache.target.trim().is_empty() {
                match std::fs::copy(format!("{}", cache.target.clone()), format!(".maid/cache/{}/target/{}", runner.name, cache.target.clone())) {
                    Ok(_) => {
                        println!("{}", format!("saved target '{}' to cache", cache.target.clone()).bright_magenta());
                        log::debug!("saved target file {}", cache.target.clone())
                    }
                    Err(err) => {
                        log::warn!("{err}");
                        crashln!("Cannot save target file.");
                    }
                };
            }
            println!("{} took {}", runner.name.white(), format!("{:.2?}", start.elapsed()).yellow());
        } else {
            println!("\n{} {} {}", helpers::string::cross_icon(), "exited with status code".bright_red(), format!("{}", exit_code).red());
            println!("{} took {}", runner.name.white(), format!("{:.2?}", start.elapsed()).yellow());
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

    run_script(
        Runner {
            name: &task.name,
            path: &task.path,
            args: &task.args,
            silent: task.silent,
            maidfile: &task.maidfile,
            script,
        },
        0,
    );
}
