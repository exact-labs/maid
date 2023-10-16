use crate::structs::Maidfile;
use colored::Colorize;
use macros_rs::{crashln, string, then};
use serde_json::json;
use std::{env, fs, io::Result, path::Path, path::PathBuf};

macro_rules! create_path {
    ($file_name:expr, $kind:expr) => {{
        let mut file_path = PathBuf::new();
        file_path.push($file_name);
        file_path.set_extension($kind);
        file_path
    }};
}

#[derive(Debug)]
struct Filesystem {
    path: Option<PathBuf>,
    is_file: bool,
}

fn working_dir() -> PathBuf {
    match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            crashln!("Unable to find current working dir");
        }
    }
}

#[allow(unused_variables)]
fn find_path(path: &Path, file_name: &str, kind: &str) -> Result<Option<fs::DirEntry>> {
    #[cfg(target_os = "linux")]
    {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = Box::leak(create_path!(file_name, kind).into_boxed_path()).to_string_lossy().to_string();

            if entry.file_name().to_string_lossy().eq_ignore_ascii_case(&file_path) {
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(None)
    }
}

fn find_file(starting_directory: &Path, file_name: &String, trace: bool) -> Option<PathBuf> {
    let mut path: PathBuf = starting_directory.into();
    let find_kind = |kind: &str, mut inner: PathBuf| -> Filesystem {
        let file_path = create_path!(file_name, kind);
        then!(working_dir() != starting_directory, inner.pop());

        match find_path(starting_directory, file_name, kind).unwrap() {
            Some(file) => inner.push(file.path()),
            None => inner.push(file_path),
        }

        if trace {
            log::trace!("{}", json!({"kind": kind, "path": inner}))
        };

        Filesystem {
            path: Some(inner.clone()),
            is_file: inner.is_file(),
        }
    };

    loop {
        for extension in vec!["", "toml", "yaml", "yml", "json", "json5"].iter() {
            let kind = find_kind(extension, path.clone());
            then!(kind.is_file, return kind.path);
        }
        then!(!path.pop(), break);
    }

    return None;
}

fn read_file(path: PathBuf, kind: &str) -> Maidfile {
    log::debug!("Maidfile is {kind}");
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{}", err);
            crashln!("Cannot find maidfile. Does it exist?");
        }
    };

    let result = match kind {
        "toml" => toml::from_str(&contents).map_err(|err| string!(err)),
        "json" => serde_json::from_str(&contents).map_err(|err| string!(err)),
        "json5" => json5::from_str(&contents).map_err(|err| string!(err)),
        "yaml" => serde_yaml::from_str(&contents).map_err(|err| string!(err)),
        _ => {
            log::warn!("Invalid format");
            crashln!("Cannot read maidfile.");
        }
    };

    match result {
        Ok(parsed) => parsed,
        Err(err) => {
            crashln!("Cannot read maidfile.\n{}", err.white());
        }
    }
}

pub fn read_maidfile_with_error(filename: &String, error: &str) -> Maidfile {
    match env::current_dir() {
        Ok(path) => match find_file(&path, &filename, true) {
            Some(path) => {
                log::info!("Found maidfile path: {}", path.display());

                let extension = path.extension().and_then(|s| s.to_str());
                match extension {
                    Some("yaml") | Some("yml") | Some("json") | Some("json5") => read_file(path.clone(), extension.unwrap()),
                    _ => read_file(path, "toml"),
                }
            }
            None => {
                log::warn!("{error}");
                crashln!("{error}");
            }
        },
        Err(err) => {
            log::warn!("{err}");
            crashln!("Home directory could not found.");
        }
    }
}

pub fn find_maidfile_root(filename: &String) -> PathBuf {
    match env::current_dir() {
        Ok(path) => match find_file(&path, &filename, false) {
            Some(mut path) => {
                path.pop();
                log::info!("Found project path: {}", path.display());
                return path;
            }
            None => {
                log::warn!("Cannot find project root.");
                crashln!("Cannot find project root.");
            }
        },
        Err(err) => {
            log::warn!("{err}");
            crashln!("Home directory could not found.");
        }
    }
}

pub fn read_maidfile(filename: &String) -> Maidfile { read_maidfile_with_error(filename, "Cannot find maidfile. Does it exist?") }
