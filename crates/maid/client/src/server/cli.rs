use crate::helpers;
use crate::server;

use colored::Colorize;
use macros_rs::{crashln, fmtstr};

pub fn connect(path: &String) {
    let values = helpers::maidfile::merge(path);
    let server = server::parse::address(values.clone());
    let token = server::parse::token(values.clone());
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
