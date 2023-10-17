use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionInfo {
    pub name: String,
    pub args: Vec<String>,
    pub remote: Remote,
    pub script: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionData {
    pub info: ConnectionInfo,
    pub maidfile: Maidfile,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Maidfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<Project>,
    pub tasks: BTreeMap<String, Tasks>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<Server>, // wip
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Server {
    pub address: Address, // wip
    pub token: String,    // wip
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Address {
    pub host: String,
    pub port: i64,
    pub ssl: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tasks {
    pub script: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<Cache>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<Remote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cache {
    pub path: String,
    pub target: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheConfig {
    pub target: Vec<String>,
    pub hash: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Remote {
    pub push: Vec<String>,
    pub pull: String,
    pub image: String,
    pub shell: String,
    pub silent: bool,
    pub exclusive: bool,
}
