use crate::helpers;
use crate::server;
use crate::structs::{Maidfile, Task, Websocket};
use crate::table;

use colored::Colorize;
use macros_rs::{crashln, fmtstr, string, then};
use reqwest::blocking::Client;
use text_placeholder::Template;
use tungstenite::{client::IntoClientRequest, connect as connectWSS, Message};

pub fn health(client: Client, values: Maidfile) -> server::api::health::Route {
    let (server, _, token, _, _) = server::parse::all(values);

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

    return body;
}

pub fn connect(path: &String) {
    let values = helpers::maidfile::merge(path);
    let client = Client::new();
    let body = health(client, values);

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
    let args = &task.args.clone();
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

    let client = Client::new();
    let body = health(client, task.maidfile.clone());
    let (server, websocket, token, host, port) = server::parse::all(task.maidfile.clone());

    crate::log!("info", "connecting to {host}:{port}");

    if body.status.healthy.data == "yes" {
        crate::log!("info", "connected successfully");
        crate::log!("notice", "{}", body.status.message.data);
    } else {
        crate::log!("warning", "failed to connect");
        crate::log!("notice", "{}", body.status.message.data);
    }

    let mut request = websocket.into_client_request().expect("Can't connect");
    request.headers_mut().insert("Authorization", fmtstr!("Bearer {token}").parse().unwrap());

    let (mut socket, response) = connectWSS(request).expect("Can't connect");
    log::debug!("response code: {}", response.status());

    let connection_data = serde_json::json!({
        "info": {
            "name": &task.name,
            "args": &task.args,
            "remote": &task.remote,
            "script": script,
        },
        "maidfile": Template::new_with_placeholder(
            &task.maidfile.clone().to_json(), "%{", "}"
        ).fill_with_hashmap(&table::create(task.maidfile.clone(), args)),
    });

    socket.send(Message::Text(serde_json::to_string(&connection_data).unwrap())).unwrap();

    loop {
        match socket.read() {
            Ok(json) => match json.to_text() {
                Ok(string) => match serde_json::from_str::<Websocket>(string) {
                    Ok(ws) => {
                        if ws.data["message"] != "" {
                            let msg = string!(ws.data["message"].as_str().unwrap());
                            crate::log!(ws.level.as_str(), "{}", msg);
                        }

                        then!(ws.data["done"] == true, break);
                    }
                    Err(err) => {
                        crate::log!("fatal", "{err}");
                    }
                },
                Err(err) => {
                    crashln!("{err}");
                }
            },
            Err(err) => {
                crashln!("{err}");
            }
        }
    }

    // run.rs:96 implement that later
    println!("\n{} {}", helpers::string::check_icon(), "finished task successfully".bright_green());
}
