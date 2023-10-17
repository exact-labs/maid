mod helpers;
mod docker;
mod globals;
mod structs;
mod table;

use futures_util::{SinkExt, StreamExt};
use macros_rs::{fmtstr, ternary};
use std::convert::Infallible;
use warp::ws::Message;
use warp::{Filter, Reply};
use bollard::Docker;
use std::env;

#[tokio::main]
async fn main() {
    globals::init();
    
    let port = 3500;
    let token = "test_token".to_string();

    let auth = warp::header::exact("Authorization", fmtstr!("Bearer {}", token));
    let health = warp::path!("api" / "health").and_then(health_handler);

    let gateway = warp::path!("ws" / "gateway").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| async {
            let (mut tx, rx) = websocket.split();
            let socket = Docker::connect_with_socket_defaults().unwrap();

            tx.send(Message::text(
                serde_json::to_string(&serde_json::json!({
                    "level": "success",
                    "time": chrono::Utc::now().timestamp_millis(),
                    "data": { "connected": true, "message": "client connected" },
                }))
                .unwrap(),
            ))
            .await
            .unwrap();
            
            docker::run::exec(tx, rx, socket).await.unwrap();
        })
    });

    let routes = health.or(gateway).and(auth);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

async fn health_handler() -> Result<impl Reply, Infallible> {
    let socket = Docker::connect_with_socket_defaults().unwrap();
    let info = socket.version().await.unwrap();
    
    let uptime = helpers::format::duration(helpers::os::uptime());
    let version = format!("Docker v{} (build {})", &info.version.clone().unwrap(), &info.git_commit.clone().unwrap());
        
    Ok(warp::reply::json(&serde_json::json!({
        "version": {
            "data": format!("v{}", env!("CARGO_PKG_VERSION")),
            "hue": "red"
        },
        "platform": {
            "data": format!("{} ({} {})", helpers::os::release(), env::consts::OS, env::consts::ARCH),
            "hue": "bright red"
        },
        "engine": {
            "data": version,
            "hue": "yellow"
        },
        "status": {
            "uptime": {
                "data": uptime,
                "hue": "green"
            },
            "healthy": {
                "data": ternary!(helpers::os::health(), "yes", "no"),
                "hue": "cyan"
            },
            "containers": {
                "data": docker::container::list(socket).await.unwrap(),
                "hue": "bright blue"
            }
        }
    })))
}
