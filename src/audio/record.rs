use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::error::AppError;
use crate::model::RecordingSession;

const DEFAULT_RECORDING_DURATION: Duration = Duration::from_secs(10);

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

    let result = record_microphone_to_wav(&temp_path, DEFAULT_RECORDING_DURATION);
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

fn record_microphone_to_wav(path: &Path, duration: Duration) -> Result<(), AppError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| AppError::Audio("no default input device available".into()))?;

    let config = device
        .default_input_config()
        .map_err(|err| AppError::Audio(format!("failed to get input config: {err}")))?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let captured_samples = Arc::new(Mutex::new(Vec::<i16>::new()));
    let err_fn = |err| eprintln!("audio input stream error: {err}");

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => {
            let capture_buffer = Arc::clone(&captured_samples);
            device.build_input_stream(
                &config.clone().into(),
                move |data: &[i16], _| {
                    if let Ok(mut buffer) = capture_buffer.lock() {
                        buffer.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let capture_buffer = Arc::clone(&captured_samples);
            device.build_input_stream(
                &config.clone().into(),
                move |data: &[u16], _| {
                    if let Ok(mut buffer) = capture_buffer.lock() {
                        buffer.extend(data.iter().map(|sample| (*sample as i32 - 32_768) as i16));
                    }
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::F32 => {
            let capture_buffer = Arc::clone(&captured_samples);
            device.build_input_stream(
                &config.clone().into(),
                move |data: &[f32], _| {
                    if let Ok(mut buffer) = capture_buffer.lock() {
                        buffer.extend(data.iter().map(|sample| {
                            let clamped = sample.clamp(-1.0, 1.0);
                            (clamped * i16::MAX as f32) as i16
                        }));
                    }
                },
                err_fn,
                None,
            )
        }
        sample_format => {
            return Err(AppError::Audio(format!(
                "unsupported sample format: {sample_format:?}"
            )))
        }
    }
    .map_err(|err| AppError::Audio(format!("failed to build input stream: {err}")))?;

    stream
        .play()
        .map_err(|err| AppError::Audio(format!("failed to start input stream: {err}")))?;

    thread::sleep(duration);
    drop(stream);

    let samples = captured_samples
        .lock()
        .map_err(|_| AppError::Audio("failed to access captured samples".into()))?;

    if samples.is_empty() {
        return Err(AppError::Audio(
            "no samples were captured from the input device".into(),
        ));
    }

    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|err| AppError::Audio(format!("failed to create wav: {err}")))?;

    for sample in samples.iter().copied() {
        writer
            .write_sample(sample)
            .map_err(|err| AppError::Audio(format!("failed to write sample: {err}")))?;
    }

    writer
        .finalize()
        .map_err(|err| AppError::Audio(format!("failed to finalize wav: {err}")))?;

    Ok(())
}