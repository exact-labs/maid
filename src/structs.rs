use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use toml::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Maidfile {
    pub import: Option<Vec<String>>,
    pub env: Option<BTreeMap<String, Value>>,
    pub project: Option<Project>,
    pub tasks: BTreeMap<String, Tasks>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: Option<String>,
    pub version: Option<String>,
    pub server: Option<Server>, // wip
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Server {
    pub address: Address,      // wip
    pub token: Option<String>, // wip
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Address {
    pub ip: String, // wip
    pub port: i64,  // wip
    pub ssl: bool,  // wip
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tasks {
    pub script: Value,
    pub retry: Option<i32>,
    pub hide: Option<bool>,
    pub path: Option<String>,
    pub info: Option<String>,
    pub cache: Option<Cache>,
    pub remote: Option<Remote>, // wip
    pub depends: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cache {
    pub path: String,
    pub target: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheConfig {
    pub target: String,
    pub hash: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Remote {
    pub push: Option<Value>,         // wip
    pub pull: Option<Value>,         // wip
    pub dependencies: Option<Value>, // wip
}

#[derive(Clone)]
pub struct Task {
    pub maidfile: Maidfile,
    pub name: String,
    pub script: Value,
    pub path: String,
    pub args: Vec<String>,
    pub silent: bool,
    pub is_dep: bool,
}

#[derive(Clone)]
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
