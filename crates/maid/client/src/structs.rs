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
    pub hide: Option<bool>,
    pub path: Option<String>,
    pub info: Option<String>,
    pub cache: Option<Cache>,
    pub remote: Option<Remote>,
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
    pub remote: Option<Remote>,
    pub script: Value,
    pub path: String,
    pub args: Vec<String>,
    pub silent: bool,
    pub is_dep: bool,
}

#[derive(Clone, Debug)]
pub struct Runner<'a> {
    pub maidfile: &'a Maidfile,
    pub name: &'a String,
    pub remote: &'a Option<Remote>,
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
