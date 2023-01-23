use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use colored::Colorize;
use just_macros::{crashln, errorln, ternary};
use serde_derive::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Instant;
use text_placeholder::Template;
use toml::Value;

#[derive(Debug, Deserialize)]
struct Maidfile {
    env: BTreeMap<String, Value>,
    tasks: BTreeMap<String, Tasks>,
}

#[derive(Debug, Deserialize)]
struct Tasks {
    script: Value,
    path: String,
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Run a task defined in maidfile
    #[arg(default_value = "", hide_default_value = true)]
    task: Vec<String>,
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(flatten)]
    verbose: Verbosity,
}

#[derive(Subcommand)]
enum Commands {
    /// List all maidfile tasks
    Tasks,
    /// Run maidfile task from path
    Run {
        #[command()]
        task: String,
        #[arg(short, long, default_value_t = String::from("maidfile"), help = "maidfile path")]
        path: String,
        #[arg(short, long, default_value = "", hide_default_value = true, help = "maidfile args")]
        args: Vec<String>,
    },
}

fn boxed(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn value_error(debug_err: &str) {
    log::debug!("unexpected {debug_err}");
    errorln!("Unable to parse maidfile. Does it contain non string values?");
}

fn exec_script(values: &Maidfile, value: &Value, path: &String, args: &Vec<String>) {
    match value {
        Value::String(string) => {
            let mut table = HashMap::new();
            for (key, value) in values.env.iter() {
                env::set_var(key, value.to_string());
                log::debug!("adding env {key} with value {}", value.to_string());
                table.insert(boxed(key.clone()), boxed(value.to_string()));
                table.insert("CWD", boxed(String::from(env::current_dir().unwrap().to_string_lossy())));

                for (pos, arg) in args.iter().enumerate() {
                    log::debug!("adding argument ${pos} with value {}", arg);
                    table.insert(boxed(format!("${pos}")), arg);
                }
            }

            let script = Template::new(string).fill_with_hashmap(&table);
            let name = script.split(" ").collect::<Vec<&str>>()[0];
            let mut args = script.split(" ").collect::<Vec<&str>>();
            args.remove(0);

            log::debug!("starting script {name} with {:?}", args);
            let mut cmd = match Command::new(name)
                .args(args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .current_dir(path)
                .spawn()
            {
                Ok(output) => output,
                Err(err) => {
                    crashln!("Cannot start command {name}.\n - {err}");
                }
            };

            let status = cmd.wait();
            let exit_code = status.as_ref().unwrap().code().unwrap();

            log::debug!("finished script {name} with exit code {:?}", exit_code);
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
                    let () = { exec_script(values, value, path, args) };
                },
            };
            result
        }

        Value::Table(_) => value_error("table"),
        Value::Integer(_) => value_error("integer"),
        Value::Float(_) => value_error("float"),
        Value::Boolean(_) => value_error("boolean"),
        Value::Datetime(_) => value_error("datetimme"),
    }
}

fn read_maidfile(path: &String) -> String {
    match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => {
            crashln!("Cannot find maidfile. Does it exist?");
        }
    }
}

fn exec_maidfile(task: &String, args: &Vec<String>, path: &String, silent: bool) {
    let start = Instant::now();
    let values: Maidfile = match toml::from_str(&read_maidfile(path)) {
        Ok(contents) => contents,
        Err(err) => {
            crashln!("Cannot read maidfile.\n - {err}");
        }
    };

    let cwd = &String::from(env::current_dir().unwrap().to_string_lossy());
    let task_path = ternary!(&values.tasks[task].path != "", &values.tasks[task].path, cwd);
    if !silent {
        let formatted_path = format!("({})", task_path.split('/').last().unwrap());
        ternary!(
            task_path == cwd,
            println!("{} {}", "»".white(), &values.tasks[task].script),
            println!("{} {} {}", formatted_path.bright_cyan(), "»".white(), &values.tasks[task].script)
        )
    }
    exec_script(&values, &values.tasks[task].script, task_path, args);
    if !silent {
        println!("\n{} {}", "✔".green(), "finished task successfully".bright_green());
        println!("{} took {}", task.white(), format!("{:.2?}", start.elapsed()).yellow());
    }
}

fn main() {
    let cli = Cli::parse();
    env_logger::Builder::new().filter_level(cli.verbose.log_level_filter()).init();

    match &cli.command {
        Some(Commands::Tasks) => println!("print all tasks"),
        Some(Commands::Run { task, path, args }) => exec_maidfile(task, args, path, cli.verbose.is_silent()),
        None => exec_maidfile(&cli.task[0], &cli.task, &String::from("maidfile"), cli.verbose.is_silent()),
    }
}
