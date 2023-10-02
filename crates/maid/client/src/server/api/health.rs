use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Value<T> {
    pub data: T,
    pub hue: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Route {
    pub uptime: Value<f32>,
    pub version: Value<String>,
    pub engine: Value<String>,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
    pub healthy: Value<String>,
    pub ping: Value<f32>,
    pub message: Value<String>,
    pub containers: Value<Vec<String>>,
}