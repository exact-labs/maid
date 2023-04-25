use crate::helpers;
use crate::structs::Maidfile;

use colored::Colorize;
use macros_rs::{errorln, ternary};
use serde_json::json;
use std::{collections::BTreeMap, collections::HashMap, env};
use text_placeholder::Template;
use toml::Value;

pub fn create(values: Maidfile, args: &Vec<String>) -> HashMap<&str, &str> {
    let mut table = HashMap::new();
    let empty_env: BTreeMap<String, Value> = BTreeMap::new();

    table.insert("os.platform", env::consts::OS);
    table.insert("os.arch", env::consts::ARCH);

    log::info!("{} os.platform: '{}'", helpers::string::add_icon(), env::consts::OS.yellow());
    log::info!("{} os.arch: '{}'", helpers::string::add_icon(), env::consts::ARCH.yellow());

    match env::current_dir() {
        Ok(path) => {
            table.insert("dir.current", helpers::string::path_to_str(&path));
            log::info!("{} dir.current: '{}'", helpers::string::add_icon(), helpers::string::path_to_str(&path).yellow());
        }
        Err(err) => {
            log::warn!("{err}");
            errorln!("Current directory could not be added as script variable.");
        }
    }

    match home::home_dir() {
        Some(path) => {
            table.insert("dir.home", helpers::string::path_to_str(&path));
            log::info!("{} dir.home: '{}'", helpers::string::add_icon(), helpers::string::path_to_str(&path).yellow());
        }
        None => {
            errorln!("Home directory could not be added as script variable.");
        }
    }

    for (pos, arg) in args.iter().enumerate() {
        log::info!("{} arg.{pos}: '{}'", helpers::string::add_icon(), arg.yellow());
        table.insert(helpers::string::to_static_str(format!("arg.{pos}")), arg);
    }

    let user_env = match &values.env {
        Some(env) => env.iter(),
        None => empty_env.iter(),
    };

    for (key, value) in user_env {
        let value_formatted = ternary!(
            value.to_string().starts_with("\""),
            helpers::string::trim_start_end(helpers::string::to_static_str(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table))),
            helpers::string::to_static_str(Template::new_with_placeholder(&value.to_string(), "%{", "}").fill_with_hashmap(&table))
        );

        env::set_var(key, value_formatted);
        log::info!("{} env.{key}: '{}'", helpers::string::add_icon(), value_formatted.yellow());
        table.insert(helpers::string::to_static_str(format!("env.{}", key.clone())), value_formatted);
    }

    log::trace!("{}", json!({ "env": table }));

    return table;
}
