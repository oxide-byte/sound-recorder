use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::error::AppError;

pub fn start_playback_thread(
    stop_flag: Arc<AtomicBool>,
    tx: mpsc::Sender<Result<(), AppError>>,
    wav_path: PathBuf,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let result = play_until_stop(stop_flag, &wav_path);
        let _ = tx.send(result);
    })
}

fn play_until_stop(stop_flag: Arc<AtomicBool>, file: &std::path::Path) -> Result<(), AppError> {
    if !file.exists() {
        return Err(AppError::InvalidArgument(format!(
            "file does not exist: {}",
            file.display()
        )));
    }

    let mut reader = hound::WavReader::open(file)
        .map_err(|err| AppError::Audio(format!("failed to open wav file: {err}")))?;
    let spec = reader.spec();
    let source_channels = spec.channels as usize;
    if source_channels == 0 {
        return Err(AppError::Audio("wav file has zero channels".into()));
    }

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
    let volume_scale = 1.0f32;

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
                        |s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16,
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
                        |s| ((s.clamp(-1.0, 1.0) * 0.5 + 0.5) * u16::MAX as f32) as u16,
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
                        |s| s.clamp(-1.0, 1.0),
                    );
                },
                err_fn,
                None,
            )
        }
        fmt => {
            return Err(AppError::Audio(format!(
                "unsupported output sample format: {fmt:?}"
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
        if finished || stop_flag.load(Ordering::Relaxed) {
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

fn read_samples_as_f32(
    reader: &mut hound::WavReader<std::io::BufReader<std::fs::File>>,
) -> Result<Vec<f32>, AppError> {
    let spec = reader.spec();
    match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Int, 16) => reader
            .samples::<i16>()
            .map(|s| {
                s.map(|v| v as f32 / i16::MAX as f32)
                    .map_err(|err| AppError::Audio(format!("failed to read wav samples: {err}")))
            })
            .collect(),
        (hound::SampleFormat::Float, 32) => reader
            .samples::<f32>()
            .map(|s| {
                s.map_err(|err| AppError::Audio(format!("failed to read wav samples: {err}")))
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
        Ok(g) => g,
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
            let src_ch = channel_index % source_channels;
            let src_idx = guard.frame_index * source_channels + src_ch;
            *out = convert(samples[src_idx] * volume_scale);
        }

        guard.frame_index += 1;
        if guard.frame_index >= guard.total_frames {
            guard.finished = true;
        }
    }
}
