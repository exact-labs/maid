use crate::structs::Maidfile;
use macros_rs::{string, ternary};

pub fn address(values: Maidfile) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => {
                let prefix = ternary!(server.address.ssl, "https", "http");
                format!("{}://{}:{}", prefix, server.address.ip, server.address.port)
            }
            None => string!(""),
        },
        None => string!(""),
    }
}

pub fn ip(values: Maidfile) -> String {
    match &values.project {
        Some(project) => match &project.server {
            Some(server) => server.address.ip.clone(),
            None => string!(""),
        },
        None => string!(""),
    }
}
