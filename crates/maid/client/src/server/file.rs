use crate::helpers;

use flate2::write::GzEncoder;
use flate2::Compression;
use global_placeholders::global;
use macros_rs::crashln;
use std::{fs::File, path::PathBuf};
use tar::Builder;
use uuid::Uuid;

fn append_to_tar(builder: &mut Builder<GzEncoder<File>>, path: &String) -> Result<(), std::io::Error> {
    let pathbuf = PathBuf::from(path);

    if pathbuf.is_file() {
        builder.append_path(&pathbuf)?;
    } else if pathbuf.is_dir() {
        builder.append_dir_all(&pathbuf, &pathbuf)?;
    }
    Ok(())
}

pub fn remove_tar(file: &String) {
    if let Err(_) = std::fs::remove_file(file) {
        crashln!("Unable to remove temporary archive. does it exist?");
    }
}

pub fn write_tar(files: &Vec<String>) -> Result<String, std::io::Error> {
    if !helpers::Exists::folder(global!("maid.temp_dir")).unwrap() {
        std::fs::create_dir_all(global!("maid.temp_dir")).unwrap();
        log::debug!("created maid temp dir");
    }

    let file_name = format!("{}/{}.tgz", global!("maid.temp_dir"), Uuid::new_v4());
    let tar_gz = File::create(&file_name)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    log::debug!("compressing to {}", &file_name);
    for path in files {
        append_to_tar(&mut tar, path)?;
        log::debug!("{} {:?}", helpers::string::add_icon(), path);
    }

    Ok(file_name)
}
