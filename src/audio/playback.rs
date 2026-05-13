use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

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

    let mut reader = hound::WavReader::open(file)
        .map_err(|err| AppError::Audio(format!("failed to open wav file: {err}")))?;
    let spec = reader.spec();
    let source_channels = spec.channels as usize;
    if source_channels == 0 {
        return Err(AppError::Audio("wav file has zero channels".into()));
    }

    let volume_scale = volume as f32 / 100.0;
    let samples = read_samples_as_f32(&mut reader)?;
    if samples.is_empty() {
        return Err(AppError::Audio("wav file contains no samples".into()));
    }

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| AppError::Audio("no default output device available".into()))?;
    let config = device
        .default_output_config()
        .map_err(|err| AppError::Audio(format!("failed to get output config: {err}")))?;

    let output_channels = config.channels() as usize;
    let state = Arc::new(Mutex::new(PlaybackState {
        frame_index: 0,
        total_frames: samples.len() / source_channels,
        finished: false,
    }));

    let err_fn = |err| eprintln!("audio output stream error: {err}");
    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => {
            let state = Arc::clone(&state);
            let samples = samples.clone();
            device.build_output_stream(
                &config.clone().into(),
                move |data: &mut [i16], _| {
                    write_output_data(
                        data,
                        &samples,
                        source_channels,
                        output_channels,
                        volume_scale,
                        &state,
                        |sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16,
                    );
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let state = Arc::clone(&state);
            let samples = samples.clone();
            device.build_output_stream(
                &config.clone().into(),
                move |data: &mut [u16], _| {
                    write_output_data(
                        data,
                        &samples,
                        source_channels,
                        output_channels,
                        volume_scale,
                        &state,
                        |sample| {
                            let scaled = sample.clamp(-1.0, 1.0);
                            ((scaled * 0.5 + 0.5) * u16::MAX as f32) as u16
                        },
                    );
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::F32 => {
            let state = Arc::clone(&state);
            let samples = samples.clone();
            device.build_output_stream(
                &config.clone().into(),
                move |data: &mut [f32], _| {
                    write_output_data(
                        data,
                        &samples,
                        source_channels,
                        output_channels,
                        volume_scale,
                        &state,
                        |sample| sample.clamp(-1.0, 1.0),
                    );
                },
                err_fn,
                None,
            )
        }
        sample_format => {
            return Err(AppError::Audio(format!(
                "unsupported output sample format: {sample_format:?}"
            )))
        }
    }
    .map_err(|err| AppError::Audio(format!("failed to build output stream: {err}")))?;

    stream
        .play()
        .map_err(|err| AppError::Audio(format!("failed to start output stream: {err}")))?;

    loop {
        let finished = state
            .lock()
            .map_err(|_| AppError::Audio("failed to access playback state".into()))?
            .finished;
        if finished {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

struct PlaybackState {
    frame_index: usize,
    total_frames: usize,
    finished: bool,
}

fn read_samples_as_f32(reader: &mut hound::WavReader<std::io::BufReader<std::fs::File>>) -> Result<Vec<f32>, AppError> {
    let spec = reader.spec();
    match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Int, 16) => reader
            .samples::<i16>()
            .map(|sample| {
                sample
                    .map(|v| v as f32 / i16::MAX as f32)
                    .map_err(|err| AppError::Audio(format!("failed to read wav samples: {err}")))
            })
            .collect(),
        (hound::SampleFormat::Float, 32) => reader
            .samples::<f32>()
            .map(|sample| {
                sample.map_err(|err| AppError::Audio(format!("failed to read wav samples: {err}")) )
            })
            .collect(),
        _ => Err(AppError::Audio(format!(
            "unsupported wav format: {:?} {}-bit",
            spec.sample_format, spec.bits_per_sample
        ))),
    }
}

fn write_output_data<T, F>(
    data: &mut [T],
    samples: &[f32],
    source_channels: usize,
    output_channels: usize,
    volume_scale: f32,
    state: &Arc<Mutex<PlaybackState>>,
    convert: F,
) where
    T: Copy,
    F: Fn(f32) -> T,
{
    if output_channels == 0 {
        return;
    }

    let mut guard = match state.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    for frame in data.chunks_mut(output_channels) {
        if guard.frame_index >= guard.total_frames {
            guard.finished = true;
            for out in frame.iter_mut() {
                *out = convert(0.0);
            }
            continue;
        }

        for (channel_index, out) in frame.iter_mut().enumerate() {
            let source_channel = channel_index % source_channels;
            let source_index = guard.frame_index * source_channels + source_channel;
            let value = samples[source_index] * volume_scale;
            *out = convert(value);
        }

        guard.frame_index += 1;
        if guard.frame_index >= guard.total_frames {
            guard.finished = true;
        }
    }
}