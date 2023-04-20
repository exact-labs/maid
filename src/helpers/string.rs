use colored::{ColoredString, Colorize};
use std::path::Path;

pub fn to_static_str(str: String) -> &'static str {
    Box::leak(str.into_boxed_str())
}

pub fn add_icon() -> ColoredString {
    "+".green()
}

pub fn path_to_str(path: &Path) -> &'static str {
    to_static_str(String::from(path.to_string_lossy()))
}

pub fn trim_start_end(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}
