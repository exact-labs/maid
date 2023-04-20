use macros_rs::{crashln, errorln};

pub fn value_error(debug_err: &str) {
    log::warn!("unexpected {debug_err} found");
    errorln!("Unable to parse maidfile. Does it contain non string values?");
}

pub fn toml_to_json<T: serde::Serialize>(values: T) -> String {
    match serde_json::to_string(&values) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot read maidfile.");
        }
    }
}

pub mod file;
pub mod string;
