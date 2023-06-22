use crate::global::Global;
use std::collections::HashMap;

pub struct GlobalRegistry;
pub static GLOBAL_REGISTRY: Global<HashMap<String, String>> = Global::new();

impl GlobalRegistry {
    pub fn get(path: String) -> String {
        let value = GLOBAL_REGISTRY.lock().unwrap();

        match value.get(&path) {
            Some(val) => val.clone(),
            None => panic!("error"),
        }
    }

    pub fn set(path: String, value: String) { GLOBAL_REGISTRY.lock_mut().unwrap().insert(path, value); }
}

#[macro_export]
macro_rules! global {
    ($path:expr $(, $args:expr)*) => {{
        let template = $crate::macros::GlobalRegistry::get($path.to_string());

        let mut result = String::new();
        let mut args_iter = vec![$($args),*].into_iter();
        let mut next_arg = || args_iter.next().unwrap_or("");
        let mut chars = template.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                if let Some(&'}') = chars.peek() {
                    chars.next();
                    result.push_str(&next_arg());
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }

        result
    }};
}

#[macro_export]
macro_rules! init {
    ($path:expr, $value:expr) => {{
        $crate::macros::GlobalRegistry::set($path.to_string(), $value.to_string())
    }};
}
