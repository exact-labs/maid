use serde_derive::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use toml::Value as TomlValue;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Maidfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, TomlValue>>,
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
    pub script: TomlValue,
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
    pub pull: Vec<String>,
    pub image: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    pub maidfile: Maidfile,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<Remote>,
    pub script: TomlValue,
    pub path: String,
    pub args: Vec<String>,
    pub silent: bool,
    pub is_dep: bool,
}

#[derive(Clone, Debug)]
pub struct Runner<'a> {
    pub maidfile: &'a Maidfile,
    pub name: &'a String,
    pub script: Vec<&'a str>,
    pub path: &'a String,
    pub args: &'a Vec<String>,
    pub silent: bool,
    pub is_dep: bool,
}

#[derive(Debug)]
pub struct DisplayTask {
    pub name: String,
    pub formatted: String,
    pub hidden: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Websocket {
    pub level: String,
    pub time: i64,
    pub data: JsonValue,
}
