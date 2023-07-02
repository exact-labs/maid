use crate::global::Global;
use std::collections::HashMap;

pub struct Registry {
    storage: Global<HashMap<String, String>>,
}

static GLOBAL: Registry = Registry { storage: Global::new() };

impl Registry {
    pub fn get(path: String) -> String {
        let value = GLOBAL.storage.lock().unwrap();

        match value.get(&path) {
            Some(val) => val.clone(),
            None => panic!("error"),
        }
    }

    pub fn set(path: String, value: String) { GLOBAL.storage.lock_mut().unwrap().insert(path, value); }
}
