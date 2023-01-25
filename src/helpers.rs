use just_macros::{crashln, errorln};
use std::{env, fs, path::Path, path::PathBuf};

pub fn find_file(starting_directory: &Path, filename: &String) -> Option<PathBuf> {
    let mut path: PathBuf = starting_directory.into();
    let file = Path::new(filename);

    loop {
        path.push(file);
        if path.is_file() {
            break Some(path);
        }
        if !(path.pop() && path.pop()) {
            break None;
        }
    }
}

pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn value_error(debug_err: &str) {
    log::warn!("unexpected {debug_err} found");
    errorln!("Unable to parse maidfile. Does it contain non string values?");
}

pub fn read_maidfile(filename: &String) -> String {
    match env::current_dir() {
        Ok(path) => match find_file(&path, &filename) {
            Some(path) => {
                log::info!("Found maidfile path: {}", path.display());
                match fs::read_to_string(path) {
                    Ok(contents) => format!("[env]\n{contents}"),
                    Err(_) => {
                        crashln!("Cannot find maidfile. Does it exist?");
                    }
                }
            }
            None => {
                crashln!("Cannot find maidfile. Does it exist?");
            }
        },
        Err(err) => {
            log::warn!("{err}");
            crashln!("Home directory could not found.");
        }
    }
}

pub fn trim_start_end(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}
