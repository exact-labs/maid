use crate::structs::Maidfile;
use macros_rs::{crashln, fmtstr, then};
use serde_json::json;
use std::{env, fs, path::Path, path::PathBuf};

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

fn find_file(starting_directory: &Path, filename: &String, trace: bool) -> Option<PathBuf> {
    let mut path: PathBuf = starting_directory.into();
    let find_kind = |kind: &str, mut inner: PathBuf| -> Filesystem {
        then!(working_dir() != starting_directory, inner.pop());
        inner.push(Path::new(fmtstr!("{filename}{kind}")));

        if trace {
            log::trace!("{}", json!({"kind": kind, "path": inner}))
        };

        return Filesystem {
            path: Some(inner.clone()),
            is_file: inner.is_file(),
        };
    };

    loop {
        let default = find_kind("", path.clone());
        let yaml = find_kind(".yaml", path.clone());
        let yml = find_kind(".yml", path.clone());
        let json = find_kind(".json", path.clone());
        let json5 = find_kind(".json5", path.clone());
        let toml = find_kind(".toml", path.clone());

        if default.is_file {
            break default.path;
        }
        if yaml.is_file {
            break yaml.path;
        }
        if yml.is_file {
            break yml.path;
        }
        if json.is_file {
            break json.path;
        }
        if json5.is_file {
            break json5.path;
        }
        if toml.is_file {
            break toml.path;
        }

        if !path.pop() {
            break None;
        }
    }
}

fn read_file(path: PathBuf, kind: &str) -> Maidfile {
    log::debug!("Maidfile is {kind}");
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot find maidfile. Does it exist?");
        }
    };

    let read_yaml = |contents: &String| -> Maidfile {
        match serde_yaml::from_str(contents) {
            Ok(contents) => contents,
            Err(err) => {
                log::warn!("{err}");
                crashln!("Cannot read maidfile.");
            }
        }
    };

    let read_json = |contents: &String| -> Maidfile {
        match serde_json::from_str(contents) {
            Ok(contents) => contents,
            Err(err) => {
                log::warn!("{err}");
                crashln!("Cannot read maidfile.");
            }
        }
    };

    let read_json5 = |contents: &String| -> Maidfile {
        match json5::from_str(contents) {
            Ok(contents) => contents,
            Err(err) => {
                log::warn!("{err}");
                crashln!("Cannot read maidfile.");
            }
        }
    };

    let read_toml = |contents: &String| -> Maidfile {
        match toml::from_str(contents) {
            Ok(contents) => contents,
            Err(err) => {
                log::warn!("{err}");
                crashln!("Cannot read maidfile.");
            }
        }
    };

    match kind {
        "yaml" => read_yaml(&contents),
        "yml" => read_yaml(&contents),
        "json" => read_json(&contents),
        "json5" => read_json5(&contents),
        "toml" => read_toml(&contents),
        _ => {
            crashln!("Cannot read maidfile.");
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
                    Some("yaml") => read_file(path, "yaml"),
                    Some("yml") => read_file(path, "yml"),
                    Some("json") => read_file(path, "json"),
                    Some("json5") => read_file(path, "json5"),
                    Some("toml") => read_file(path, "toml"),
                    Some(_) => read_file(path, "toml"),
                    None => read_file(path, "toml"),
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
