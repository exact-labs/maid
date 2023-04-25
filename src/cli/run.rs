use crate::cli;
use crate::helpers;
use crate::shell::IntoArgs;
use colored::Colorize;
use macros_rs::{crashln, inc};
use serde_json::json;
use std::io::Error;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Instant;
use text_placeholder::Template;
use toml::Value;

fn run(values: &cli::Maidfile, task: &String, scripts: Vec<&str>, path: &String, args: &Vec<String>, silent: bool, mut retry_times: i32) {
    let start = Instant::now();
    let mut status_array: Vec<Result<ExitStatus, Error>> = vec![];

    let retry = match &values.tasks[task].retry {
        Some(retry) => *retry,
        None => 0,
    };

    for string in scripts.clone() {
        let start = Instant::now();
        let table = cli::create_table(values.clone(), args);
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
            .current_dir(path)
            .spawn()
        {
            Ok(output) => output, // add silent no output mode when in deps unless verbose
            Err(err) => {
                log::warn!("{err}");
                crashln!("Cannot start command {name}.");
            }
        };

        let status = cmd.wait();
        let exit_code = helpers::exit_code(&status);
        let success = helpers::success(&status);

        if !success && retry >= 1 {
            print!("\n");

            if retry > retry_times {
                inc!(retry_times);
                println!("{} {} {}", "⚠".yellow(), "retry attempt".bright_yellow(), format!("{retry_times}").yellow());
                run(values, task, scripts.clone(), path, &args, silent, retry_times);
            }
        }

        status_array.push(status);
        log::debug!("Finished cmd: '{name} {}' with exit code: {:?} in {:.2?}", args.join(" "), exit_code, start.elapsed());
    }

    let status = match status_array.last() {
        Some(status) => status,
        None => crashln!("Failed to fetch final status code."),
    };

    let exit_code = helpers::exit_code(status);
    let success = helpers::success(&status);

    if !success && retry > retry_times && retry >= 1 {
        crashln!(
            "{} {} {}",
            helpers::string::cross_icon(),
            "max retries reached, exited with status code".bright_red(),
            format!("{}", exit_code).red()
        );
    }

    if !silent && retry_times < 1 {
        if success {
            println!("\n{} {}", helpers::string::check_icon(), "finished task successfully".bright_green());
            println!("{} took {}", task.white(), format!("{:.2?}", start.elapsed()).yellow());
        } else {
            println!("\n{} {} {}", helpers::string::cross_icon(), "exited with status code".bright_red(), format!("{}", exit_code).red());
            println!("{} took {}", task.white(), format!("{:.2?}", start.elapsed()).yellow());
        }
    }
}

pub fn task(values: &cli::Maidfile, task: &String, value: &Value, path: &String, args: &Vec<String>, silent: bool) {
    let mut script: Vec<&str> = vec![];

    if value.is_str() {
        match value.as_str() {
            Some(cmd) => script.push(cmd),
            None => crashln!("Unable to parse maidfile. Missing string value."),
        };
    } else if value.is_array() {
        match IntoIterator::into_iter(match value.as_array() {
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
        helpers::value_error(value.type_str())
    }

    run(values, task, script, path, args, silent, 0);
}
