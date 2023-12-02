mod docker;
mod globals;
mod helpers;
mod structs;
mod table;

use bollard::{Docker, API_DEFAULT_VERSION};
use docker::container;
use macros_rs::{fmtstr, ternary};
use rocket::futures::SinkExt;
use rocket::{get, http::Status, launch, outcome::Outcome, routes, State};
use rocket_ws::{Channel, Message, WebSocket};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

struct DockerState {
    docker: Result<Docker, anyhow::Error>,
}

#[derive(Serialize, Deserialize)]
enum Level {
    None,
    Fatal,
    Docker,
    Debug,
    Error,
    Notice,
    Info,
    Build,
    Warning,
    Success,
}

#[derive(Serialize, Deserialize)]
enum Kind {
    Done,
    Binary,
    Message,
}

struct Response {
    level: Level,
    kind: Kind,
    message: Option<String>,
}

impl Response {
    fn to_string(&self) -> String {
        let json_value = serde_json::json!({
            "kind": &self.kind,
            "level": &self.level,
            "message": &self.message,
            "time": chrono::Utc::now().timestamp_millis(),
        });

        serde_json::to_string(&json_value).unwrap()
    }
}

impl From<Response> for Message {
    fn from(response: Response) -> Self { Message::text(response.to_string()) }
}

#[derive(Debug)]
struct Token(String);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for Token {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let token = "test_token1".to_string();
        let authorization_header = request.headers().get_one("Authorization");

        if let Some(header_value) = authorization_header {
            if header_value == fmtstr!("Bearer {token}") {
                let token = header_value.trim_start_matches("Bearer ").to_owned();
                return Outcome::Success(Token(token));
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}

#[get("/api/health")]
async fn health(docker_state: &State<DockerState>, _token: Token) -> Value {
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

#[get("/ws/gateway")]
fn stream(ws: WebSocket, docker_state: &State<DockerState>, _token: Token) -> Channel {
    let connect_success = Response {
        level: Level::Success,
        kind: Kind::Message,
        message: Some("client connected".to_string()),
    };

    ws.channel(move |mut stream| {
        Box::pin(async move {
            stream.send(connect_success.into()).await?;

            match docker::run::exec(stream, &docker_state.docker).await {
                Ok(_) => log::info!("build finished"),
                Err(_) => log::error!("failed to build"),
            };

            Ok(())
        })
    })
}

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    let http = true;

    std::env::set_var("ROCKET_PORT", "3500");
    std::env::set_var("RUST_LOG", "info");

    globals::init();
    env_logger::init();

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
