use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::AppError;
use crate::model::RecordingSession;

const SAMPLE_RATE: u32 = 44_100;
const CHANNELS: u16 = 1;

pub fn validate_record_request(output: &str) -> Result<(), AppError> {
    if output.trim().is_empty() {
        return Err(AppError::InvalidArgument("output path cannot be empty".into()));
    }

    let output_path = Path::new(output);
    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        return Err(AppError::InvalidArgument(format!(
            "output directory does not exist: {}",
            parent.display()
        )));
    }

    Ok(())
}

pub fn record_to_wav(output: &str, _input_device_id: Option<&str>) -> Result<(), AppError> {
    validate_record_request(output)?;

    let output_path = PathBuf::from(output);
    let temp_path = output_path.with_extension("tmp.wav");

    let result = write_silence_wav(&temp_path, Duration::from_millis(250));
    if let Err(err) = result {
        let _ = fs::remove_file(&temp_path);
        return Err(err);
    }

    fs::rename(&temp_path, &output_path)?;

    let _session = RecordingSession {
        input_device_id: None,
        output_path: output.to_string(),
        status: "completed".to_string(),
    };

    Ok(())
}

fn write_silence_wav(path: &Path, duration: Duration) -> Result<(), AppError> {
    let spec = hound::WavSpec {
        channels: CHANNELS,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let sample_count = (duration.as_secs_f32() * SAMPLE_RATE as f32) as usize;
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|err| AppError::Audio(format!("failed to create wav: {err}")))?;

    for _ in 0..sample_count {
        writer
            .write_sample::<i16>(0)
            .map_err(|err| AppError::Audio(format!("failed to write sample: {err}")))?;
    }

    writer
        .finalize()
        .map_err(|err| AppError::Audio(format!("failed to finalize wav: {err}")))?;

    Ok(())
}