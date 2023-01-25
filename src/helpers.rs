use just_macros::{crashln, errorln};
use std::fs;

pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn value_error(debug_err: &str) {
    log::warn!("unexpected {debug_err} found");
    errorln!("Unable to parse maidfile. Does it contain non string values?");
}

pub fn read_maidfile(path: &String) -> String {
    match fs::read_to_string(path) {
        Ok(contents) => format!("[env]\n{contents}"),
        Err(_) => {
            crashln!("Cannot find maidfile. Does it exist?");
        }
    }
}

pub fn trim_start_end(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}
