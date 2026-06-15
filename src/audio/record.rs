use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::error::AppError;
use crate::model::{AudioOutputProfile, CompressionProfile, SupportedFormat};

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
    profile: AudioOutputProfile,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let result = record_until_stop(stop_flag, &recordings_dir, profile);
        let _ = tx.send(result);
    })
}

fn record_until_stop(
    stop_flag: Arc<AtomicBool>,
    recordings_dir: &Path,
    profile: AudioOutputProfile,
) -> Result<PathBuf, AppError> {
    let filename = generate_wav_filename();
    let final_path = recordings_dir.join(&filename);
    let temp_path = final_path.with_extension("tmp.wav");

    let result = record_microphone_until_stop(Arc::clone(&stop_flag), &temp_path, profile);
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
    profile: AudioOutputProfile,
) -> Result<(), AppError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| AppError::Audio("no default input device available".into()))?;

    let config = device
        .default_input_config()
        .map_err(|err| AppError::Audio(format!("failed to get input config: {err}")))?;

    let sample_rate = config.sample_rate();
    let channels = config.channels();
    let captured_samples = Arc::new(Mutex::new(Vec::<i16>::new()));
    let err_fn = |err| eprintln!("audio input stream error: {err}");

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => {
            let buf = Arc::clone(&captured_samples);
            device.build_input_stream(
                config.clone().into(),
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
                config.clone().into(),
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
                config.clone().into(),
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

    write_samples_to_wav(path, &samples, sample_rate, channels, profile)
}

/// Writes a buffer of `i16` samples to a WAV file using the bit-depth/sample-format
/// dispatch table from `specs/004-audio-format-compression/contracts/output-profile.md`.
pub fn write_samples_to_wav(
    path: &Path,
    samples: &[i16],
    sample_rate: u32,
    channels: u16,
    profile: AudioOutputProfile,
) -> Result<(), AppError> {
    match profile.format {
        SupportedFormat::Wav => {}
    }

    let (sample_format, bits_per_sample) = match profile.compression {
        CompressionProfile::Pcm8 => (hound::SampleFormat::Int, 8u16),
        CompressionProfile::Pcm16 => (hound::SampleFormat::Int, 16u16),
        CompressionProfile::Pcm24 => (hound::SampleFormat::Int, 24u16),
        CompressionProfile::Float32 => (hound::SampleFormat::Float, 32u16),
    };

    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample,
        sample_format,
    };

    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|err| AppError::Audio(format!("failed to create wav: {err}")))?;

    match profile.compression {
        CompressionProfile::Pcm8 => {
            for &s in samples {
                let s8 = (s >> 8) as i8;
                writer
                    .write_sample(s8)
                    .map_err(|err| AppError::Audio(format!("failed to write sample: {err}")))?;
            }
        }
        CompressionProfile::Pcm16 => {
            for &s in samples {
                writer
                    .write_sample(s)
                    .map_err(|err| AppError::Audio(format!("failed to write sample: {err}")))?;
            }
        }
        CompressionProfile::Pcm24 => {
            for &s in samples {
                let s24 = (s as i32) << 8;
                writer
                    .write_sample(s24)
                    .map_err(|err| AppError::Audio(format!("failed to write sample: {err}")))?;
            }
        }
        CompressionProfile::Float32 => {
            for &s in samples {
                let sf = s as f32 / i16::MAX as f32;
                writer
                    .write_sample(sf)
                    .map_err(|err| AppError::Audio(format!("failed to write sample: {err}")))?;
            }
        }
    }

    writer
        .finalize()
        .map_err(|err| AppError::Audio(format!("failed to finalize wav: {err}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sound_recorder_record_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    fn synth_samples() -> Vec<i16> {
        (0..2048i32).map(|n| ((n * 31) as i16).wrapping_mul(2)).collect()
    }

    fn write_and_read_spec(profile: AudioOutputProfile) -> hound::WavSpec {
        let path = temp_path(&format!("{}_{}.wav", profile.format.as_id(), profile.compression.as_id()));
        let samples = synth_samples();
        write_samples_to_wav(&path, &samples, 44_100, 1, profile).unwrap();
        let reader = hound::WavReader::open(&path).unwrap();
        let spec = reader.spec();
        let _ = std::fs::remove_file(&path);
        spec
    }

    #[test]
    fn writer_dispatches_pcm8() {
        let spec = write_and_read_spec(AudioOutputProfile {
            format: SupportedFormat::Wav,
            compression: CompressionProfile::Pcm8,
        });
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);
        assert_eq!(spec.bits_per_sample, 8);
    }

    #[test]
    fn writer_dispatches_pcm16() {
        let spec = write_and_read_spec(AudioOutputProfile::default());
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);
        assert_eq!(spec.bits_per_sample, 16);
    }

    #[test]
    fn writer_dispatches_pcm24() {
        let spec = write_and_read_spec(AudioOutputProfile {
            format: SupportedFormat::Wav,
            compression: CompressionProfile::Pcm24,
        });
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);
        assert_eq!(spec.bits_per_sample, 24);
    }

    #[test]
    fn writer_dispatches_float32() {
        let spec = write_and_read_spec(AudioOutputProfile {
            format: SupportedFormat::Wav,
            compression: CompressionProfile::Float32,
        });
        assert_eq!(spec.sample_format, hound::SampleFormat::Float);
        assert_eq!(spec.bits_per_sample, 32);
    }

    #[test]
    fn writer_preserves_sample_rate_and_channels() {
        let path = temp_path("preserve.wav");
        let samples = synth_samples();
        write_samples_to_wav(&path, &samples, 48_000, 2, AudioOutputProfile::default()).unwrap();
        let reader = hound::WavReader::open(&path).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_rate, 48_000);
        let _ = std::fs::remove_file(&path);
    }
}