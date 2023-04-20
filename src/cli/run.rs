use crate::cli;
use crate::helpers;
use crate::shell::IntoArgs;
use macros_rs::crashln;
use std::process::{Command, Stdio};
use text_placeholder::Template;
use toml::Value;

pub fn task(values: &cli::Maidfile, value: &Value, path: &String, args: &Vec<String>) {
    match value {
        Value::String(string) => {
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
