mod commands;
mod errors;
mod generator;
mod logging;
mod templates;
use commands::create_module;
use errors::CliError;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "asterisk console",
    version = "0.1.0",
    about = "Code generator for asterisk application",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct asteriskCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new module (backend + migration)
    CreateModule,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let cli = asteriskCli::parse();

    match cli.command {
        Commands::CreateModule => create_module::run(),
    }
}
