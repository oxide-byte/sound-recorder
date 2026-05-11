mod audio;
mod cli;
mod error;
mod model;
mod tui;

use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::error::AppError;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ListDevices => cli::commands::list_devices(),
        Commands::Record {
            output,
            input_device,
        } => cli::commands::record(output, input_device),
        Commands::Play {
            file,
            output_device,
            volume,
        } => cli::commands::play(file, output_device, volume),
        Commands::Tui => tui::run_tui(),
    }
}
