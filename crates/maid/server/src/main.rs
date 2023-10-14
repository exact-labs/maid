use futures_util::{SinkExt, StreamExt};
use macros_rs::fmtstr;
use std::convert::Infallible;
use std::time::Duration;
use warp::ws::Message;
use warp::{Filter, Reply};

#[tokio::main]
async fn main() {
    let port = 3500;
    let token = "test_token".to_string();

    let log = warp::log("maid");
    let auth = warp::header::exact("Authorization", fmtstr!("Bearer {}", token));
    let health = warp::path!("api" / "health").and_then(health_handler);

    let gateway = warp::path!("ws" / "gateway").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| async {
            let (mut tx, mut rx) = websocket.split();

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

            tokio::time::sleep(Duration::from_millis(1000)).await;

            tx.send(Message::text(
                serde_json::to_string(&serde_json::json!({
                    "level": "warning",
                    "time": chrono::Utc::now().timestamp_millis(),
                    "data": { "message": "some warning idk" },
                }))
                .unwrap(),
            ))
            .await
            .unwrap();

            tokio::time::sleep(Duration::from_millis(2500)).await;

            tx.send(Message::binary(tokio::fs::read("../testing/test.tgz").await.unwrap())).await.unwrap();

            tx.send(Message::text(
                serde_json::to_string(&serde_json::json!({
                    "level": "success",
                    "time": chrono::Utc::now().timestamp_millis(),
                    "data": { "done": true, "message": "" },
                }))
                .unwrap(),
            ))
            .await
            .unwrap();

            while let Some(result) = rx.next().await {
                let message = result.unwrap();
                println!("received message: {:?}", message);
            }
        })
    });

    let routes = health.or(gateway).with(log).and(auth);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

async fn health_handler() -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&serde_json::json!({
        "uptime": {
            "data": 168.44,
            "hue": "red"
        },
        "version": {
            "data": "0.2.1",
            "hue": "bright red"
        },
        "engine": {
            "data": "docker",
            "hue": "yellow"
        },
        "status": {
            "ping": {
                "data": 36,
                "hue": "green"
            },
            "healthy": {
                "data": "yes",
                "hue": "cyan"
            },
            "message": {
                "data": "all services running",
                "hue": "bright blue"
            },
            "containers": {
                "data": ["build", "build/ui"],
                "hue": "magenta"
            }
        }
    })))
}
