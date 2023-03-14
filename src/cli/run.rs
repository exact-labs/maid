use crate::cli;
use crate::helpers;
use crate::shell::IntoArgs;
use colored::Colorize;
use just_macros::{crashln, errorln, ternary};
use std::process::{Command, Stdio};
use std::{collections::HashMap, env};
use text_placeholder::Template;
use toml::Value;

pub fn task(values: &cli::Maidfile, value: &Value, path: &String, args: &Vec<String>) {
    match value {
        Value::String(string) => {
            let mut table = HashMap::new();

            table.insert("os.platform", env::consts::OS);
            table.insert("os.arch", env::consts::ARCH);

            log::info!("{} os.platform: '{}'", helpers::add_icon(), env::consts::OS.yellow());
            log::info!("{} os.arch: '{}'", helpers::add_icon(), env::consts::ARCH.yellow());

            match env::current_dir() {
                Ok(path) => {
                    table.insert("dir.current", helpers::path_to_str(&path));
                    log::info!("{} dir.current: '{}'", helpers::add_icon(), helpers::path_to_str(&path).yellow());
                }
                Err(err) => {
                    log::warn!("{err}");
                    errorln!("Current directory could not be added as script variable.");
                }
            }

            match home::home_dir() {
                Some(path) => {
                    table.insert("dir.home", helpers::path_to_str(&path));
                    log::info!("{} dir.home: '{}'", helpers::add_icon(), helpers::path_to_str(&path).yellow());
                }
                None => {
                    errorln!("Home directory could not be added as script variable.");
                }
            }

            for (pos, arg) in args.iter().enumerate() {
                log::info!("{} arg.{pos}: '{}'", helpers::add_icon(), arg.yellow());
                table.insert(helpers::string_to_static_str(format!("arg.{pos}")), arg);
            }

            for (key, value) in values.env.iter() {
                let value_formatted = ternary!(
                    value.to_string().starts_with("\""),
                    helpers::trim_start_end(helpers::string_to_static_str(Template::new_with_placeholder(&value.to_string(), "{", "}").fill_with_hashmap(&table))),
                    helpers::string_to_static_str(Template::new_with_placeholder(&value.to_string(), "{", "}").fill_with_hashmap(&table))
                );

                env::set_var(key, value_formatted);
                log::info!("{} env.{key}: '{}'", helpers::add_icon(), value_formatted.yellow());
                table.insert(helpers::string_to_static_str(format!("env.{}", key.clone())), value_formatted);
            }

            let script = Template::new_with_placeholder(string, "{", "}").fill_with_hashmap(&table);
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
            log::trace!("Command name: {name}");
            log::trace!("Command args: {:?}", args);

            log::info!("Execute Command: '{name} {}'", args.join(" "));
            let mut cmd = match Command::new(&name)
                .args(args.clone())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .current_dir(path)
                .spawn()
            {
                Ok(output) => output,
                Err(err) => {
                    log::warn!("{err}");
                    crashln!("Cannot start command {name}.");
                }
            };

            let status = cmd.wait();
            let exit_code = status.as_ref().unwrap().code().unwrap();

            log::debug!("Finished script: '{name} {}' with exit code: {:?}", args.join(" "), exit_code);
            if !status.as_ref().unwrap().success() {
                crashln!("✖ exited with status code {}", exit_code);
            }
        }

        Value::Array(array) => {
            let result = match IntoIterator::into_iter(array.iter()) {
                mut iter => loop {
                    let next;
                    match Iterator::next(&mut iter) {
                        Option::Some(val) => next = val,
                        Option::None => break,
                    };
                    let value = next;
                    let () = { task(values, value, path, args) };
                },
            };
            result
        }

        Value::Table(_) => helpers::value_error("table"),
        Value::Integer(_) => helpers::value_error("integer"),
        Value::Float(_) => helpers::value_error("float"),
        Value::Boolean(_) => helpers::value_error("boolean"),
        Value::Datetime(_) => helpers::value_error("datetimme"),
    }
}
