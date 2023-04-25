use optional_field::Field;
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
    pub server: Option<Server>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Server {
    pub address: Address,
    pub token: Field<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tasks {
    pub script: Value,
    pub retry: Option<i32>,
    pub hide: Option<bool>,
    pub cache: Field<bool>,
    pub path: Field<String>,
    pub info: Field<String>,
    pub target: Field<Value>,
    pub remote: Field<Remote>,
    pub depends: Field<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Address {
    pub ip: Field<String>,
    pub port: Field<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Remote {
    pub push: Field<Value>,
    pub pull: Field<Value>,
    pub worker: Field<String>,
    pub dependencies: Field<Value>,
}

#[derive(Clone)]
pub struct Task {
    pub maidfile: Maidfile,
    pub name: String,
    pub script: Value,
    pub path: String,
    pub args: Vec<String>,
    pub silent: bool,
}

#[derive(Clone)]
pub struct Runner<'a> {
    pub maidfile: &'a Maidfile,
    pub name: &'a String,
    pub script: Vec<&'a str>,
    pub path: &'a String,
    pub args: &'a Vec<String>,
    pub silent: bool,
}

#[derive(Debug)]
pub struct DisplayTask {
    pub name: String,
    pub formatted: String,
    pub hidden: bool,
}
