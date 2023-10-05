use colored::{ColoredString, Colorize};
use std::path::Path;

pub fn warn_icon() -> ColoredString { "⚠".yellow() }
pub fn seperator() -> ColoredString { ":".white() }
pub fn arrow_icon() -> ColoredString { "»".white() }
pub fn add_icon() -> ColoredString { "+".green() }
pub fn cross_icon() -> ColoredString { "✖".red() }
pub fn check_icon() -> ColoredString { "✔".green() }

pub fn path_to_str(path: &Path) -> &'static str { Box::leak(String::from(path.to_string_lossy()).into_boxed_str()) }

pub fn trim_start_end(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}
