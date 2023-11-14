use chrono::Datelike;
use std::env;
use std::process::Command;

struct Env {}
impl Env {
    fn git() {
        let output = Command::new("git").args(&["rev-parse", "--short=10", "HEAD"]).output().unwrap();
        println!("cargo:rustc-env=GIT_HASH={}", String::from_utf8(output.stdout).unwrap());

        let output_full = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
        println!("cargo:rustc-env=GIT_HASH_FULL={}", String::from_utf8(output_full.stdout).unwrap());
    }

    fn date() {
        let date = chrono::Utc::now();
        println!("cargo:rustc-env=BUILD_DATE={}-{}-{}", date.year(), date.month(), date.day());
    }

    fn misc() {
        println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap());
    }
}

fn main() {
    Env::git();
    Env::date();
    Env::misc();
}
