use crate::helpers;
use crate::parse;
use crate::structs::{DisplayTask, Maidfile};

use macros_rs::crashln;

pub fn merge(path: &String) -> Maidfile {
    let mut values = helpers::file::read_maidfile(path);
    let imported_values = parse::import::push(values.import.clone());

    for import in imported_values.iter() {
        values = match merge_struct::merge(&values, &import) {
            Ok(merge) => merge,
            Err(err) => {
                log::warn!("{err}");
                crashln!("Unable to import tasks.");
            }
        };
    }

    return values;
}

impl Maidfile {
    pub fn to_json(&self) -> String {
        match serde_json::to_string(&self) {
            Ok(contents) => contents,
            Err(err) => {
                log::warn!("{err}");
                crashln!("Cannot read maidfile.");
            }
        }
    }
}

impl std::fmt::Display for DisplayTask {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.formatted, f)
    }
}
