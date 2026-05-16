use std::fs;
use std::io;
use std::path::Path;

use crate::error::AppError;
use crate::model::{AudioOutputProfile, CompressionProfile, SupportedFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    File,
    Fallback,
}

#[derive(Debug, Clone)]
pub struct AudioDefaultsConfig {
    pub profile: AudioOutputProfile,
    pub source: ConfigSource,
}

/// Parses the on-disk config-file format documented in
/// `specs/004-audio-format-compression/contracts/config-file.md` and returns
/// a validated `AudioDefaultsConfig` with `source = File`.
///
/// `Fallback` is only produced by `load_or_default` when the file is missing.
pub fn parse_config(text: &str) -> Result<AudioDefaultsConfig, AppError> {
    let mut format_entry: Option<(String, usize)> = None;
    let mut compression_entry: Option<(String, usize)> = None;

    for (idx, raw) in text.lines().enumerate() {
        let lineno = idx + 1;
        let trimmed = raw.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some(eq_pos) = trimmed.find('=') else {
            let preview: String = trimmed.chars().take(80).collect();
            return Err(AppError::Config(format!(
                "line {lineno}: expected 'key=value', got '{preview}'"
            )));
        };

        let key = trimmed[..eq_pos].trim();
        let value = trimmed[eq_pos + 1..].trim();

        if key.is_empty() {
            return Err(AppError::Config(format!("line {lineno}: empty key")));
        }
        if value.is_empty() {
            return Err(AppError::Config(format!(
                "line {lineno}: empty value for key '{key}'"
            )));
        }

        match key.to_ascii_lowercase().as_str() {
            "format" => {
                if let Some((_, prior)) = &format_entry {
                    return Err(AppError::Config(format!(
                        "line {lineno}: duplicate key 'format' (already set on line {prior})"
                    )));
                }
                format_entry = Some((value.to_string(), lineno));
            }
            "compression" => {
                if let Some((_, prior)) = &compression_entry {
                    return Err(AppError::Config(format!(
                        "line {lineno}: duplicate key 'compression' (already set on line {prior})"
                    )));
                }
                compression_entry = Some((value.to_string(), lineno));
            }
            _ => {
                return Err(AppError::Config(format!(
                    "line {lineno}: unknown key '{key}' (supported: format, compression)"
                )));
            }
        }
    }

    let (format_raw, _) = format_entry.ok_or_else(|| {
        AppError::Config("missing required key 'format' in config/audio.conf".to_string())
    })?;
    let (compression_raw, _) = compression_entry.ok_or_else(|| {
        AppError::Config("missing required key 'compression' in config/audio.conf".to_string())
    })?;

    let format = SupportedFormat::from_id(&format_raw)?;
    let compression = CompressionProfile::from_id(&compression_raw)?;
    let profile = AudioOutputProfile::validated(format, compression)?;

    Ok(AudioDefaultsConfig {
        profile,
        source: ConfigSource::File,
    })
}

/// Loads audio defaults from `path`. Missing file → built-in defaults with
/// `source = Fallback`. Other I/O failures bubble up as `AppError::Io`.
pub fn load_or_default(path: &Path) -> Result<AudioDefaultsConfig, AppError> {
    match fs::read_to_string(path) {
        Ok(text) => parse_config(&text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(AudioDefaultsConfig {
            profile: AudioOutputProfile::default(),
            source: ConfigSource::Fallback,
        }),
        Err(e) => Err(AppError::Io(e)),
    }
}
