use crate::cli::Maidfile;
use macros_rs::{crashln, fmtstr, then};
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

fn find_file(starting_directory: &Path, filename: &String) -> Option<PathBuf> {
    let mut path: PathBuf = starting_directory.clone().into();
    let find_kind = |kind: &str, mut inner: PathBuf| -> Filesystem {
        then!(working_dir() != starting_directory, inner.pop());
        inner.push(Path::new(fmtstr!("{filename}{kind}")));

        // println!("Finding path: {{\"kind\": \"{kind}\", \"path\":\"{}\"}}", inner.display());
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
        if toml.is_file {
            break toml.path;
        }

        if !(path.pop() && path.pop()) {
            break None;
        }
    }
}

fn read_file(path: PathBuf, kind: &str, format: &str) -> Maidfile {
    log::debug!("Maidfile is {kind}");
    let contents = match fs::read_to_string(path) {
        Ok(contents) => format!("{format}{contents}"),
        Err(err) => {
            log::warn!("{err}");
            crashln!("Cannot find maidfile. Does it exist?");
        }
    };

    let read_yml = |contents: &String| -> Maidfile {
        match serde_yaml::from_str(contents) {
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
        "yaml" => read_yml(&contents),
        "yml" => read_yml(&contents),
        "json" => read_yml(&contents),
        "toml" => read_toml(&contents),
        _ => {
            crashln!("Cannot read maidfile.");
        }
    }
}

pub fn get_current_working_dir() -> String {
    match env::current_dir() {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(_) => {
            crashln!("Unable to find current working dir");
        }
    }
}

pub fn read_maidfile(filename: &String) -> Maidfile {
    match env::current_dir() {
        Ok(path) => match find_file(&path, &filename) {
            Some(path) => {
                log::info!("Found maidfile path: {}", path.display());

                let extension = path.extension().and_then(|s| s.to_str());
                match extension {
                    Some("yaml") => read_file(path, "yaml", ""),
                    Some("yml") => read_file(path, "yml", ""),
                    Some("json") => read_file(path, "json", ""),
                    Some("toml") => read_file(path, "toml", "[env]\n"),
                    Some(_) => read_file(path, "toml", "[env]\n"),
                    None => read_file(path, "toml", "[env]\n"),
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
