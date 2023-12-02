#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {{
        lazy_static::lazy_static! {
            static ref LEVEL_COLORS: std::collections::HashMap<Level, (&'static str, &'static str)> = {
                let mut map = std::collections::HashMap::new();
                map.insert(Level::Fatal, ("FATAL", "bright red"));
                map.insert(Level::Docker, ("DOCKER", "bright yellow"));
                map.insert(Level::Info, ("INFO", "cyan"));
                map.insert(Level::Build, ("BUILD", "bright green"));
                map.insert(Level::Success, ("SUCCESS", "green"));
                map.insert(Level::Debug, ("DEBUG", "magenta"));
                map.insert(Level::Notice, ("NOTICE", "bright blue"));
                map.insert(Level::Warning, ("WARN", "yellow"));
                map.insert(Level::Error, ("ERROR", "red"));
                return map;
            };
        }

        if $level == Level::None {
            print!("{}", format_args!($($arg)*).to_string());
        } else {
            match LEVEL_COLORS.get(&$level) {
                Some((level_text, color_func)) => {
                    let level_text = level_text.color(color_func.to_string());
                    println!("{} {}", level_text, format_args!($($arg)*).to_string())
                }
                None => println!("Unknown log level: {:?}", $level),
            };
        }
    }};
}
