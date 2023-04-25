use macros_rs::{crashln, errorln};
use std::{io::Error, process::ExitStatus};

pub fn value_error(debug_err: &str) {
    errorln!("Unable to parse maidfile. Contains unexpected {debug_err} values.");
}

pub fn struct_to_json<T: serde::Serialize>(values: T) -> String {
    match serde_json::to_string(&values) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot read maidfile.");
        }
    }
}

pub fn exit_code(status: &Result<ExitStatus, Error>) -> i32 {
    match status.as_ref() {
        Ok(status) => match status.code() {
            Some(iter) => iter,
            None => crashln!("Missing status value"),
        },
        Err(err) => {
            log::warn!("{err}");
            crashln!("Unknown error, check verbose.");
        }
    }
}

pub fn success(status: &Result<ExitStatus, Error>) -> bool {
    match status.as_ref() {
        Ok(status) => status.success(),
        Err(err) => {
            log::warn!("{err}");
            crashln!("Unknown error, check verbose.");
        }
    }
}

pub mod file;
pub mod string;
