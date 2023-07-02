#[macro_export]
macro_rules! init {
    ($path:expr, $value:expr) => {{
        $crate::registry::Registry::set($path.to_string(), $value.to_string())
    }};
}

#[macro_export]
macro_rules! global {
    ($path:expr $(, $args:expr)*) => {{
        let template = $crate::registry::Registry::get($path.to_string());

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
