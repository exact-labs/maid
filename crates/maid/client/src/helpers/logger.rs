#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        let level_colors: std::collections::HashMap<&str, (&str, &str)> = [
            ("fatal", ("FATAL", "bright red")),
            ("error", ("ERROR", "red")),
            ("warning", ("WARN", "yellow")),
            ("success", ("SUCCESS", "green")),
            ("notice", ("NOTICE", "bright blue")),
            ("info", ("INFO", "cyan")),
            ("debug", ("DEBUG", "magenta")),
        ]
        .iter()
        .cloned()
        .collect();

        match level_colors.get($level) {
            Some((level_text, color_func)) => {
                let level_text = level_text.color(color_func.to_string());
                println!("{} {}", level_text, format_args!($($arg)*).to_string())
            }
            None => println!("Unknown log level: {}", $level),
        }
    };
}
