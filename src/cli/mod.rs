pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "sound-recorder")]
#[command(about = "Terminal audio recorder/player with deterministic CLI output")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List available input/output audio devices.
    ListDevices,
    /// Record audio to a WAV file.
    Record {
        #[arg(long)]
        output: String,
        #[arg(long)]
        input_device: Option<String>,
    },
    /// Play a WAV file.
    Play {
        #[arg(long)]
        file: String,
        #[arg(long)]
        output_device: Option<String>,
        #[arg(long, default_value_t = 100)]
        volume: u8,
    },
    /// Launch interactive Ratatui mode.
    Tui,
}