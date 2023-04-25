mod cli;
mod helpers;
mod parse;
mod shell;
mod structs;
mod table;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

// add remote build server - dependencies required for builds (build requires clean etc)

#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Run a task defined in maidfile
    #[arg(default_value = "", hide_default_value = true)]
    task: Vec<String>,
    #[arg(global = true, short, long, default_value_t = String::from("maidfile"), help = "maidfile path")]
    path: String,
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(flatten)]
    verbose: Verbosity,
}

#[derive(Subcommand)]
enum Commands {
    /// All internal maid commands
    Butler {
        #[command(subcommand)]
        internal: Butler,
    },
}

#[derive(Subcommand)]
enum Butler {
    /// List all maidfile tasks
    Tasks,
    /// Get Project Info
    Info,
    /// Test server specified in maidfile
    Connect,
    /// Clear maid cache
    Clean,
    /// Return the maidfile in json
    Json {
        #[arg(long, default_value_t = false, help = "Hydrate json output with env")]
        hydrate: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    env_logger::Builder::new().filter_level(cli.verbose.log_level_filter()).init();

    match &cli.command {
        Some(Commands::Butler { internal }) => match internal {
            Butler::Json { hydrate } => cli::tasks::json(&cli.path, &cli.task, hydrate),
            Butler::Info => cli::info(&cli.path),
            Butler::Connect => println!("test server here"),
            Butler::Clean => println!("delete maid cache"),
            Butler::Tasks => cli::tasks::list(&cli.path, cli.verbose.is_silent(), cli.verbose.log_level()),
        },
        None => cli::exec(cli.task[0].trim(), &cli.task, &cli.path, cli.verbose.is_silent(), cli.verbose.log_level()),
    }
}
