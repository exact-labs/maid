use crate::api;
use crate::helpers;
use crate::server;

use colored::Colorize;
use macros_rs::{crashln, ternary};

pub fn connect(path: &String) {
    let values = helpers::maidfile::merge(path);
    let server = server::parse::address(values);
    let test_path = format!("{server}/api/health");

    let res = match reqwest::blocking::get(test_path) {
        Ok(res) => res,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Unable to connect to the maid server. Is it up?");
        }
    };

    let body = match res.json::<api::health::Route>() {
        Ok(body) => body,
        Err(err) => crashln!("{err}"),
    };

    println!(
        "{}\n{}\n{}\n{}",
        "Server Info".green().bold(),
        format!(" {}: {}", "- Uptime".white(), body.uptime.red()),
        format!(" {}: {}", "- Version".white(), format!("v{}", body.version).bright_red()),
        format!(" {}: {}", "- Engine".white(), body.engine.yellow()),
    );

    println!(
        "{}\n{}\n{}\n{}\n{}",
        "Server Status".green().bold(),
        format!(" {}: {}", "- Ping".white(), format!("{}ms", body.status.ping).green()),
        format!(" {}: {}", "- Healthy".white(), ternary!(body.status.healthy, "yes".cyan(), "no".red())),
        format!(" {}: {}", "- Message".white(), body.status.message.bright_blue()),
        format!(" {}: {}", "- Containers".white(), format!("{:?}", body.status.containers).magenta()),
    );
}
