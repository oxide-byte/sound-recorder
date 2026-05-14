use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::error::AppError;

pub fn ensure_recordings_dir(dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dir)?;
    Ok(())
}

pub fn generate_wav_filename() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();

    let time_of_day = secs % 86400;
    let hh = time_of_day / 3600;
    let mm = (time_of_day % 3600) / 60;
    let ss = time_of_day % 60;

    let days = secs / 86400;
    let (year, month, day) = days_to_ymd(days);

    format!(
        "recording_{year:04}{month:02}{day:02}_{hh:02}{mm:02}{ss:02}_{millis:03}.wav"
    )
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u64, m as u64, d as u64)
}

pub fn start_recording_thread(
    stop_flag: Arc<AtomicBool>,
    tx: mpsc::Sender<Result<PathBuf, AppError>>,
    recordings_dir: PathBuf,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let result = record_until_stop(stop_flag, &recordings_dir);
        let _ = tx.send(result);
    })
}

fn record_until_stop(
    stop_flag: Arc<AtomicBool>,
    recordings_dir: &Path,
) -> Result<PathBuf, AppError> {
    let filename = generate_wav_filename();
    let final_path = recordings_dir.join(&filename);
    let temp_path = final_path.with_extension("tmp.wav");

    let result = record_microphone_until_stop(Arc::clone(&stop_flag), &temp_path);
    if let Err(err) = result {
        let _ = fs::remove_file(&temp_path);
        return Err(err);
    }

    fs::rename(&temp_path, &final_path)?;
    Ok(final_path)
}

fn record_microphone_until_stop(
    stop_flag: Arc<AtomicBool>,
    path: &Path,
) -> Result<(), AppError> {
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
            let buf = Arc::clone(&captured_samples);
            device.build_input_stream(
                &config.clone().into(),
                move |data: &[i16], _| {
                    if let Ok(mut b) = buf.lock() {
                        b.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let buf = Arc::clone(&captured_samples);
            device.build_input_stream(
                &config.clone().into(),
                move |data: &[u16], _| {
                    if let Ok(mut b) = buf.lock() {
                        b.extend(data.iter().map(|s| (*s as i32 - 32_768) as i16));
                    }
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::F32 => {
            let buf = Arc::clone(&captured_samples);
            device.build_input_stream(
                &config.clone().into(),
                move |data: &[f32], _| {
                    if let Ok(mut b) = buf.lock() {
                        b.extend(data.iter().map(|s| {
                            (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
                        }));
                    }
                },
                err_fn,
                None,
            )
        }
        fmt => {
            return Err(AppError::Audio(format!(
                "unsupported sample format: {fmt:?}"
            )))
        }
    }
    .map_err(|err| AppError::Audio(format!("failed to build input stream: {err}")))?;

    stream
        .play()
        .map_err(|err| AppError::Audio(format!("failed to start input stream: {err}")))?;

    while !stop_flag.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(50));
    }
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
