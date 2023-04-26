use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Route {
    pub uptime: String,
    pub version: String,
    pub engine: String,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
    pub healthy: bool,
    pub ping: i32,
    pub message: String,
    pub containers: Vec<String>,
}
