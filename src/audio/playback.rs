use std::path::Path;

use crate::error::AppError;

pub fn play_wav(file: &str, _output_device_id: Option<&str>, volume: u8) -> Result<(), AppError> {
    if !Path::new(file).exists() {
        return Err(AppError::InvalidArgument(format!(
            "file does not exist: {file}"
        )));
    }

    if volume > 100 {
        return Err(AppError::InvalidArgument(format!(
            "volume must be between 0 and 100, got {volume}"
        )));
    }

    Ok(())
}