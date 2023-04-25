use macros_rs::{crashln, errorln};
use std::{io::Error, process::ExitStatus};

pub fn error(debug_err: &str) {
    errorln!("Unable to parse maidfile. Contains unexpected {debug_err} values.");
}

pub fn code(status: &Result<ExitStatus, Error>) -> i32 {
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
