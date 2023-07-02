use anyhow::Error;
use macros_rs::str;
use std::path::Path;

pub struct Exists;
impl Exists {
    pub fn folder(dir_name: String) -> Result<bool, Error> {
        Ok(Path::new(str!(dir_name)).is_dir())
    }
    pub fn file(file_name: String) -> Result<bool, Error> {
        Ok(Path::new(str!(file_name)).exists())
    }
}

pub mod file;
pub mod maidfile;
pub mod status;
pub mod string;
