use crate::structs::Maidfile;
use macros_rs::{string, ternary};

pub fn address(values: &Maidfile) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => {
                let prefix = ternary!(server.address.ssl, "https", "http");
                format!("{}://{}:{}", prefix, server.address.host, server.address.port)
            }
            None => string!(""),
        },
        None => string!(""),
    }
}

pub fn websocket(values: &Maidfile) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => {
                let prefix = ternary!(server.address.ssl, "wss", "ws");
                format!("{}://{}:{}/ws/gateway", prefix, server.address.host, server.address.port)
            }
            None => string!(""),
        },
        None => string!(""),
    }
}

pub fn host(values: &Maidfile) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => server.address.host.clone(),
            None => string!(""),
        },
        None => string!(""),
    }
}

pub fn port(values: &Maidfile) -> i64 {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => server.address.port.clone(),
            None => 0,
        },
        None => 0,
    }
}

pub fn token(values: &Maidfile) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => server.token.clone(),
            None => string!(""),
        },
        None => string!(""),
    }
}

pub fn all(maidfile: Maidfile) -> (String, String, String, String, i64) { (address(&maidfile), websocket(&maidfile), token(&maidfile), host(&maidfile), port(&maidfile)) }
