use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Value<T> {
    pub data: T,
    pub hue: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Route {
    pub platform: Value<String>,
    pub version: Value<String>,
    pub engine: Value<String>,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Status {
    pub uptime: Value<String>,
    pub healthy: Value<String>,
    pub containers: Value<Vec<String>>,
}
