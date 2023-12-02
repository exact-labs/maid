mod docker;
mod globals;
mod helpers;
mod structs;
mod table;

use bollard::{Docker, API_DEFAULT_VERSION};
use docker::container;
use macros_rs::ternary;
use rocket::{get, launch, routes, State};
use rocket_ws::{Config, Stream, WebSocket};
use serde_json::{json, Value};
use std::env;

struct DockerState {
    docker: Result<Docker, anyhow::Error>,
}

#[get("/api/health")]
async fn health(docker_state: &State<DockerState>) -> Value {
    let socket = &docker_state.docker.as_ref().unwrap();
    let info = socket.version().await.unwrap();
    let containers = container::list(socket).await.unwrap();

    let uptime = helpers::format::duration(helpers::os::uptime());
    let version = format!("Docker v{} (build {})", &info.version.clone().unwrap(), &info.git_commit.clone().unwrap());

    json!({
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
                "data": containers,
                "hue": "bright blue"
            }
        }
    })
}

#[get("/echo")]
fn stream(ws: WebSocket) -> Stream!['static] {
    let ws = ws.config(Config {
        max_send_queue: Some(5),
        ..Default::default()
    });

    Stream! { ws =>
        for await message in ws {
            yield message?;
        }
    }
}

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    globals::init();

    let http = true;
    let token = "test_token".to_string();

    std::env::set_var("ROCKET_PORT", "3500");

    let socket = async move {
        let socket = match http {
            true => Docker::connect_with_http("100.79.107.11:4250", 120, API_DEFAULT_VERSION)?.clone(),
            false => Docker::connect_with_socket_defaults()?.clone(),
        };

        Ok::<Docker, anyhow::Error>(socket)
    };

    let docker_socket = tokio::spawn(socket);
    let docker_socket = docker_socket.await.unwrap();

    rocket::build().manage(DockerState { docker: docker_socket }).mount("/", routes![health, stream])
}

// #[tokio::main]
// async fn main() ->  {
//     // let connection = Docker::connect_with_http(
//     //                    "http://my-custom-docker-server:2735", 4, API_DEFAULT_VERSION)
//     //                    .unwrap();
//
//     // let session = SessionBuilder::default()
//     //     .user("root".to_string())
//     //     .port(22)
//     //     .known_hosts_check(KnownHosts::Accept)
//     //     .control_directory(std::env::temp_dir())
//     //     .connect_timeout(Duration::from_secs(5))
//     //     .connect("100.79.107.11")
//     //     .await;
//
//     let auth = warp::header::exact("Authorization", fmtstr!("Bearer {}", token));
//     let health = warp::path!("api" / "health").and_then(health_handler);
//
//
//
//
//
//     let routes = health.or(gateway).and(auth);
//
//     Ok(warp::serve(routes).run(([0, 0, 0, 0], port)).await)
// }
