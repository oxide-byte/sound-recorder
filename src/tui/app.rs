use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::error::AppError;

pub fn run() -> Result<(), AppError> {
    enable_raw_mode().map_err(AppError::Io)?;
    disable_raw_mode().map_err(AppError::Io)?;
    println!("tui mode started");
    Ok(())
}