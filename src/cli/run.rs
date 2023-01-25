use crate::cli;
use crate::helpers;
use just_macros::{crashln, errorln};
use std::process::{Command, Stdio};
use std::{collections::HashMap, env};
use text_placeholder::Template;
use toml::Value;

pub fn task(values: &cli::Maidfile, value: &Value, path: &String, args: &Vec<String>) {
    match value {
        Value::String(string) => {
            let mut table = HashMap::new();

            match env::current_dir() {
                Ok(path) => {
                    table.insert("CWD", helpers::string_to_static_str(String::from(path.to_string_lossy())));
                    log::info!("Added working dir: {}", path.display());
                }
                Err(err) => {
                    log::warn!("{err}");
                    errorln!("Home directory could not be added as script variable.");
                }
            }

            match home::home_dir() {
                Some(path) => {
                    table.insert("HOME", helpers::string_to_static_str(String::from(path.to_string_lossy())));
                    log::info!("Added home dir: {}", path.display());
                }
                None => {
                    errorln!("Home directory could not be added as script variable.");
                }
            }

            for (key, value) in values.env.iter() {
                env::set_var(key, value.to_string());
                log::debug!("Adding env: {key} with value: {}", helpers::trim_start_end(&value.to_string()));
                table.insert(helpers::string_to_static_str(key.clone()), helpers::trim_start_end(helpers::string_to_static_str(value.to_string())));
            }

            for (pos, arg) in args.iter().enumerate() {
                log::debug!("Adding argument: ${pos} with value: {}", arg);
                table.insert(helpers::string_to_static_str(format!("${pos}")), arg);
            }

            let script = Template::new(string).fill_with_hashmap(&table);
            let name = script.split(" ").collect::<Vec<&str>>()[0];
            let mut args = script.split(" ").collect::<Vec<&str>>();
            args.remove(0);

            log::info!("Execute Command: '{name} {}'", args.join(" "));
            let mut cmd = match Command::new(name)
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

            log::info!("Finished script: '{name} {}' with exit code: {:?}", args.join(" "), exit_code);
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
