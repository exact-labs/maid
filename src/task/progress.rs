use indicatif::{ProgressBar, ProgressStyle};
use macros_rs::fmtstr;

pub fn init(ticks: Vec<&str>, template: &str, tick: u64) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    let tick_str: Vec<&str> = ticks.into_iter().map(|item| fmtstr!("{item} ")).collect();

    pb.enable_steady_tick(std::time::Duration::from_millis(tick));
    pb.set_style(ProgressStyle::with_template(template).unwrap().tick_strings(&*tick_str));

    return pb;
}
