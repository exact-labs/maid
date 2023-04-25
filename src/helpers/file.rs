use macros_rs::crashln;
use std::env;

pub fn get_current_working_dir() -> String {
    match env::current_dir() {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(_) => {
            crashln!("Unable to find current working dir");
        }
    }
}
