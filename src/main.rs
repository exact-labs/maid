use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use colored::Colorize;
use just_macros::{crashln, errorln};
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
    tasks: BTreeMap<String, Value>,
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Run a task defined in maidfile
    #[arg(default_value = "", hide_default_value = true)]
    task: String,
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
    },
}

fn boxed(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn exec_script(values: &Maidfile, value: &Value) {
    match value {
        Value::String(string) => {
            let mut table = HashMap::new();
            for (key, value) in values.env.iter() {
                env::set_var(key, value.to_string());
                table.insert(boxed(key.clone()), boxed(value.to_string()));
            }

            let script = Template::new(string).fill_with_hashmap(&table);
            let name = script.split(" ").collect::<Vec<&str>>()[0];
            let mut args = script.split(" ").collect::<Vec<&str>>();
            args.remove(0);

            let mut cmd = Command::new(name).args(args).stdout(Stdio::inherit()).stderr(Stdio::inherit()).spawn().unwrap();
            let _ = cmd.wait();
        }
        Value::Integer(_) => {
            errorln!("Unable to parse maidfile. Does it contain non string values?");
        }
        Value::Float(_) => {
            errorln!("Unable to parse maidfile. Does it contain non string values?");
        }
        Value::Boolean(_) => {
            errorln!("Unable to parse maidfile. Does it contain non string values?");
        }
        Value::Datetime(_) => {
            errorln!("Unable to parse maidfile. Does it contain non string values?");
        }
        Value::Array(array) => {
            for v in array.iter() {
                exec_script(values, v);
            }
        }
        Value::Table(table) => {
            for (_, v) in table.iter() {
                exec_script(values, v);
            }
        }
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

fn exec_maidfile(task: &String, path: &String) {
    let start = Instant::now();
    let values: Maidfile = toml::from_str(&read_maidfile(path)).expect("Should have been able to read the file");
    println!("{} {}\n", "»".white(), &values.tasks[task]["script"]);
    exec_script(&values, &values.tasks[task]);
    println!("\n{} took {}", task.white(), format!("{:.2?}", start.elapsed()).yellow())
}

fn main() {
    let cli = Cli::parse();
    env_logger::Builder::new().filter_level(cli.verbose.log_level_filter()).init();

    match &cli.command {
        Some(Commands::Tasks) => println!("print all tasks"),
        Some(Commands::Run { task, path }) => exec_maidfile(task, path),
        None => exec_maidfile(&cli.task, &String::from("maidfile")),
    }
}
