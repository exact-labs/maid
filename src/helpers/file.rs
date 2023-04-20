use crate::cli::Maidfile;
use just_macros::crashln;
use std::{env, fs, path::Path, path::PathBuf};

fn find_file(starting_directory: &Path, filename: &String) -> Option<PathBuf> {
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
