use crate::helpers;
use crate::server;
use crate::structs::{ConnectionData, ConnectionInfo, Maidfile, Task, Websocket};

use colored::Colorize;
use macros_rs::{crashln, fmtstr, then};
use reqwest::blocking::Client;
use tungstenite::protocol::frame::{coding::CloseCode::Normal, CloseFrame};
use tungstenite::{client::connect_with_config, client::IntoClientRequest, protocol::WebSocketConfig, Message};

fn health(client: Client, values: Maidfile) -> server::api::health::Route {
    let address = server::parse::address(&values);
    let token = server::parse::token(&values);

    let response = match client.get(fmtstr!("{address}/api/health")).header("Authorization", fmtstr!("Bearer {token}")).send() {
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
        format!(" {}: {}", "- Version".white(), body.version.data.color(body.version.hue)),
        format!(" {}: {}", "- Platform".white(), body.platform.data.color(body.platform.hue)),
        format!(" {}: {}", "- Engine".white(), body.engine.data.color(body.engine.hue)),
    );

    println!(
        "{}\n{}\n{}\n{}",
        "Server Status".green().bold(),
        format!(" {}: {}", "- Uptime".white(), body.status.uptime.data.color(body.status.uptime.hue)),
        format!(" {}: {}", "- Healthy".white(), body.status.healthy.data.color(body.status.healthy.hue)),
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

    let client = Client::new();
    let body = health(client, task.maidfile.clone());
    let (_, websocket, token, host, port) = server::parse::all(task.maidfile.clone());

    crate::log!("info", "connecting to {host}:{port}");

    if body.status.healthy.data == "yes" {
        crate::log!("notice", "server reports healthy");
    } else {
        crate::log!("warning", "failed to connect");
    }

    let websocket_config = WebSocketConfig {
        max_frame_size: Some(314572800),
        ..Default::default()
    };

    let mut request = websocket.into_client_request().expect("Can't connect");
    request.headers_mut().insert("Authorization", fmtstr!("Bearer {token}").parse().unwrap());

    let (mut socket, response) = connect_with_config(request, Some(websocket_config), 3).expect("Can't connect");
    log::debug!("response code: {}", response.status());

    let connection_data = ConnectionData {
        info: ConnectionInfo {
            name: task.name.clone(),
            args: task.args.clone(),
            remote: task.remote.clone().unwrap(),
            script: script.clone().iter().map(|&s| s.to_string()).collect(),
        },
        maidfile: task.maidfile.clone(),
    };

    let file_name = match server::file::write_tar(&task.remote.unwrap().push) {
        Ok(name) => name,
        Err(err) => {
            crashln!("Unable to create archive.\nError: {err}")
        }
    };

    log::debug!("sending information");
    socket.send(Message::Text(serde_json::to_string(&connection_data).unwrap())).unwrap();

    loop {
        match socket.read() {
            Ok(Message::Text(text)) => {
                if let Ok(Websocket { time: _, data, level }) = serde_json::from_str::<Websocket>(&text) {
                    data.get("message").and_then(|m| m.as_str()).map(|msg| {
                        if !msg.is_empty() {
                            crate::log!(level.as_str(), "{}", msg);
                        }
                    });

                    if data.get("binary").map_or(false, |d| d.as_bool().unwrap_or(false)) {
                        log::debug!("sending archive");
                        socket.send(Message::Binary(std::fs::read(&file_name).unwrap())).unwrap();
                    }

                    then!(data.get("done").map_or(false, |d| d.as_bool().unwrap_or(false)), break);
                }
            }
            Ok(Message::Binary(archive)) => {
                let archive_name = match server::file::read_tar(&archive) {
                    Ok(name) => name,
                    Err(err) => {
                        crashln!("Unable to read archive.\nError: {err}")
                    }
                };

                if let Err(err) = server::file::unpack_tar(&archive_name) {
                    crashln!("Unable to create archive.\nError: {err}")
                }

                server::file::remove_tar(&archive_name);
            }
            Err(err) => {
                crate::log!("fatal", "{err}");
                break;
            }
            _ => (),
        };
    }

    server::file::remove_tar(&file_name);
    // run.rs:96 implement that later
    println!("\n{} {}", helpers::string::check_icon(), "finished task successfully".bright_green());
    println!("{}", "removed temporary archive".bright_magenta());

    if let Err(err) = socket.close(Some(CloseFrame {
        code: Normal,
        // run.rs:96 implement that later
        reason: std::borrow::Cow::Borrowed("finished task successfully"),
    })) {
        crashln!("Unable to close socket.\nError: {err}")
    };
}
