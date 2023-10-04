use crate::helpers;
use crate::server;
use crate::structs::{Runner, Task};

use colored::Colorize;
use macros_rs::{crashln, fmtstr};

pub fn connect(path: &String) {
    let values = helpers::maidfile::merge(path);
    let (_, server, _, token) = server::parse::all(values);
    let client = reqwest::blocking::Client::new();

    let response = match client.get(fmtstr!("{server}/api/health")).header("Authorization", fmtstr!("Bearer {token}")).send() {
        Ok(res) => res,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Unable to connect to the maid server. Is it up?");
        }
    };

    let body = match response.json::<server::api::health::Route>() {
        Ok(body) => body,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Unable to connect to the maid server. Is the token correct?")
        }
    };

    println!(
        "{}\n{}\n{}\n{}",
        "Server Info".green().bold(),
        format!(" {}: {}", "- Uptime".white(), format!("{}d", body.uptime.data).color(body.uptime.hue)),
        format!(" {}: {}", "- Version".white(), format!("v{}", body.version.data).color(body.version.hue)),
        format!(" {}: {}", "- Engine".white(), body.engine.data.color(body.engine.hue)),
    );

    println!(
        "{}\n{}\n{}\n{}\n{}",
        "Server Status".green().bold(),
        format!(" {}: {}", "- Ping".white(), format!("{}ms", body.status.ping.data).color(body.status.ping.hue)),
        format!(" {}: {}", "- Healthy".white(), body.status.healthy.data.color(body.status.healthy.hue)),
        format!(" {}: {}", "- Message".white(), body.status.message.data.color(body.status.message.hue)),
        format!(" {}: {}", "- Containers".white(), format!("{:?}", body.status.containers.data).color(body.status.containers.hue)),
    );
}

pub fn remote(task: Task) {
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

    let (host, server, websocket, token) = server::parse::all(task.maidfile.clone());
    let client = reqwest::blocking::Client::new();

    println!("{host}\n{server}\n{token}\n{websocket}");

    println!(
        "{:#?}",
        Runner {
            name: &task.name,
            path: &task.path,
            args: &task.args,
            silent: task.silent,
            remote: &task.remote,
            is_dep: task.is_dep,
            maidfile: &task.maidfile,
            script,
        }
    );
}
