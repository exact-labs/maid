mod cli;
mod helpers;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Run a task defined in maidfile
    #[arg(default_value = "", hide_default_value = true)]
    task: Vec<String>,
    #[arg(short, long, default_value_t = String::from("maidfile"), help = "maidfile path")]
    path: String,
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(flatten)]
    verbose: Verbosity,
}

#[derive(Subcommand)]
enum Commands {
    /// List all maidfile tasks
    Tasks,
}

fn main() {
    let cli = Cli::parse();
    env_logger::Builder::new().filter_level(cli.verbose.log_level_filter()).init();

    match &cli.command {
        Some(Commands::Tasks) => cli::tasks::list(&cli.path, cli.verbose.is_silent(), cli.verbose.log_level()),
        None => cli::exec(&cli.task[0], &cli.task, &cli.path, cli.verbose.is_silent(), cli.verbose.log_level()),
    }
}
