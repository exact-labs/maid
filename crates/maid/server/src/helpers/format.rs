pub fn duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    
    if days == 0 {
        format!("{:02}:{:02}:{:02}", hours % 24, minutes % 60, seconds % 60)
    } else {
        format!("{} days {:02}:{:02}:{:02}", days, hours % 24, minutes % 60, seconds % 60)
    }
}