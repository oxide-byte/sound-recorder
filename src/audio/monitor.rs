use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::audio::record::{generate_wav_filename, write_samples_to_wav};
use crate::error::AppError;
use crate::model::{AudioOutputProfile, MonitorEvent, MonitoringSubState};

// ── Configuration ─────────────────────────────────────────────────────────────

pub struct MonitorConfig {
    /// Fraction of i16::MAX used as the sound-detection amplitude cutoff.
    pub threshold_fraction: f32,
    /// How long silence must persist after sound before a segment is finalized.
    pub silence_timeout: Duration,
    /// Length of the rolling pre-roll buffer prepended to each new segment.
    pub pre_roll: Duration,
    /// Minimum finalized segment duration; shorter segments are discarded.
    pub min_clip_duration: Duration,
    /// Output format/compression applied to each finalized segment.
    pub output_profile: AudioOutputProfile,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            threshold_fraction: 0.01,
            silence_timeout: Duration::from_millis(1500),
            pre_roll: Duration::from_millis(400),
            min_clip_duration: Duration::from_millis(500),
            output_profile: AudioOutputProfile::default(),
        }
    }
}

// ── Internal types ─────────────────────────────────────────────────────────────

struct PreRollBuffer {
    buf: VecDeque<i16>,
    capacity: usize,
}

impl PreRollBuffer {
    fn new(sample_rate: u32, channels: u16, pre_roll: Duration) -> Self {
        let capacity =
            (sample_rate as f64 * channels as f64 * pre_roll.as_secs_f64()).ceil() as usize;
        Self {
            buf: VecDeque::with_capacity(capacity + 1),
            capacity,
        }
    }

    fn push_batch(&mut self, batch: &[i16]) {
        self.buf.extend(batch.iter().copied());
        while self.buf.len() > self.capacity {
            self.buf.pop_front();
        }
    }

    fn drain(&mut self) -> Vec<i16> {
        self.buf.drain(..).collect()
    }
}

struct SoundSegment {
    samples: Vec<i16>,
    sample_rate: u32,
    channels: u16,
    start_timestamp: String,
}

impl SoundSegment {
    fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            samples: Vec::new(),
            sample_rate,
            channels,
            start_timestamp: generate_wav_filename(),
        }
    }
}

// ── Pure helpers ──────────────────────────────────────────────────────────────

fn peak_amplitude(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let max_abs = samples.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
    max_abs as f32 / i16::MAX as f32
}

fn drain_buffer(buf: &Arc<Mutex<Vec<i16>>>) -> Vec<i16> {
    buf.lock()
        .map(|mut guard| std::mem::take(&mut *guard))
        .unwrap_or_default()
}

// ── Segment finalization ──────────────────────────────────────────────────────

fn finalize_or_discard(
    seg: SoundSegment,
    config: &MonitorConfig,
    recordings_dir: &Path,
    tx: &mpsc::Sender<MonitorEvent>,
) {
    let duration_secs =
        seg.samples.len() as f64 / (seg.sample_rate as f64 * seg.channels as f64);
    let min_secs = config.min_clip_duration.as_secs_f64();

    if duration_secs < min_secs {
        let ms = (duration_secs * 1000.0).round() as u64;
        let _ = tx.send(MonitorEvent::SegmentDiscarded {
            reason: format!("segment too short ({ms}ms)"),
        });
        let _ = tx.send(MonitorEvent::SubStateChanged(MonitoringSubState::Listening));
        return;
    }

    let final_path = recordings_dir.join(&seg.start_timestamp);
    let temp_path = final_path.with_extension("tmp.wav");

    match write_samples_to_wav(
        &temp_path,
        &seg.samples,
        seg.sample_rate,
        seg.channels,
        config.output_profile,
    ) {
        Ok(()) => match fs::rename(&temp_path, &final_path) {
            Ok(()) => {
                let _ = tx.send(MonitorEvent::SegmentSaved(final_path));
                let _ = tx.send(MonitorEvent::SubStateChanged(MonitoringSubState::Listening));
            }
            Err(e) => {
                let _ = fs::remove_file(&temp_path);
                let _ = tx.send(MonitorEvent::Failed(AppError::Io(e)));
            }
        },
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            let _ = tx.send(MonitorEvent::Failed(e));
        }
    }
}

// ── Thread entry point ────────────────────────────────────────────────────────

pub fn start_monitoring_thread(
    stop_flag: Arc<AtomicBool>,
    tx: mpsc::Sender<MonitorEvent>,
    recordings_dir: PathBuf,
    config: MonitorConfig,
) -> JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = run_monitor(stop_flag, &tx, &recordings_dir, &config) {
            let _ = tx.send(MonitorEvent::Failed(e));
        }
    })
}

const CONTINUOUS_TRIGGER_SECS: u64 = 30;
const POLL_INTERVAL: Duration = Duration::from_millis(50);

fn run_monitor(
    stop_flag: Arc<AtomicBool>,
    tx: &mpsc::Sender<MonitorEvent>,
    recordings_dir: &Path,
    config: &MonitorConfig,
) -> Result<(), AppError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| AppError::Audio("no default input device available".into()))?;
    let stream_config = device
        .default_input_config()
        .map_err(|e| AppError::Audio(format!("failed to get input config: {e}")))?;

    let sample_rate = stream_config.sample_rate();
    let channels = stream_config.channels();
    let shared_buf: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));

    let stream = {
        let buf = Arc::clone(&shared_buf);
        let tx_err = tx.clone();
        match stream_config.sample_format() {
            cpal::SampleFormat::I16 => device.build_input_stream(
                &stream_config.clone().into(),
                move |data: &[i16], _| {
                    if let Ok(mut b) = buf.lock() {
                        b.extend_from_slice(data);
                    }
                },
                move |e| {
                    let _ = tx_err
                        .send(MonitorEvent::Failed(AppError::Audio(format!("stream error: {e}"))));
                },
                None,
            ),
            cpal::SampleFormat::U16 => {
                let tx_err2 = tx.clone();
                device.build_input_stream(
                    &stream_config.clone().into(),
                    move |data: &[u16], _| {
                        if let Ok(mut b) = buf.lock() {
                            b.extend(data.iter().map(|s| (*s as i32 - 32_768) as i16));
                        }
                    },
                    move |e| {
                        let _ = tx_err2.send(MonitorEvent::Failed(AppError::Audio(format!(
                            "stream error: {e}"
                        ))));
                    },
                    None,
                )
            }
            cpal::SampleFormat::F32 => {
                let tx_err3 = tx.clone();
                device.build_input_stream(
                    &stream_config.clone().into(),
                    move |data: &[f32], _| {
                        if let Ok(mut b) = buf.lock() {
                            b.extend(
                                data.iter()
                                    .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16),
                            );
                        }
                    },
                    move |e| {
                        let _ = tx_err3.send(MonitorEvent::Failed(AppError::Audio(format!(
                            "stream error: {e}"
                        ))));
                    },
                    None,
                )
            }
            fmt => {
                return Err(AppError::Audio(format!("unsupported sample format: {fmt:?}")))
            }
        }
        .map_err(|e| AppError::Audio(format!("failed to build input stream: {e}")))?
    };

    stream
        .play()
        .map_err(|e| AppError::Audio(format!("failed to start input stream: {e}")))?;

    let mut pre_roll = PreRollBuffer::new(sample_rate, channels, config.pre_roll);
    let mut current_segment: Option<SoundSegment> = None;
    let mut last_above_threshold: Option<Instant> = None;
    let mut capture_start: Option<Instant> = None;
    let mut continuous_trigger_sent = false;

    loop {
        thread::sleep(POLL_INTERVAL);

        if stop_flag.load(Ordering::Relaxed) {
            let batch = drain_buffer(&shared_buf);
            if let Some(seg) = current_segment.as_mut() {
                if !batch.is_empty() {
                    seg.samples.extend_from_slice(&batch);
                }
            }
            if let Some(seg) = current_segment.take() {
                finalize_or_discard(seg, config, recordings_dir, tx);
            }
            break;
        }

        let batch = drain_buffer(&shared_buf);
        if batch.is_empty() {
            continue;
        }

        let peak = peak_amplitude(&batch);

        if let Some(seg) = current_segment.as_mut() {
            seg.samples.extend_from_slice(&batch);
            if peak >= config.threshold_fraction {
                last_above_threshold = Some(Instant::now());
            }

            let silence_expired = last_above_threshold
                .map(|t| t.elapsed() >= config.silence_timeout)
                .unwrap_or(false);

            if silence_expired {
                let seg = current_segment.take().unwrap();
                capture_start = None;
                last_above_threshold = None;
                continuous_trigger_sent = false;
                finalize_or_discard(seg, config, recordings_dir, tx);
                continue;
            }

            if let Some(cs) = capture_start {
                if !continuous_trigger_sent
                    && cs.elapsed() >= Duration::from_secs(CONTINUOUS_TRIGGER_SECS)
                {
                    let _ = tx.send(MonitorEvent::ContinuousTriggering);
                    continuous_trigger_sent = true;
                }
            }
        } else {
            pre_roll.push_batch(&batch);
            if peak >= config.threshold_fraction {
                let pre_roll_samples = pre_roll.drain();
                let mut seg = SoundSegment::new(sample_rate, channels);
                seg.samples.extend_from_slice(&pre_roll_samples);
                current_segment = Some(seg);
                last_above_threshold = Some(Instant::now());
                capture_start = Some(Instant::now());
                continuous_trigger_sent = false;
                let _ = tx.send(MonitorEvent::SubStateChanged(MonitoringSubState::Capturing));
            }
        }
    }

    drop(stream);
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::mpsc;

    fn make_temp_dir() -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "sound_recorder_test_{}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos(),
            n
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    // ── peak_amplitude ────────────────────────────────────────────────────────

    #[test]
    fn test_peak_amplitude_empty_is_zero() {
        assert_eq!(peak_amplitude(&[]), 0.0);
    }

    #[test]
    fn test_peak_amplitude_above_default_threshold() {
        let config = MonitorConfig::default();
        let threshold_sample = (config.threshold_fraction * i16::MAX as f32) as i16 + 100;
        let samples = vec![threshold_sample, -threshold_sample];
        assert!(peak_amplitude(&samples) >= config.threshold_fraction);
    }

    #[test]
    fn test_peak_amplitude_below_default_threshold() {
        let config = MonitorConfig::default();
        let threshold_sample = (config.threshold_fraction * i16::MAX as f32) as i16;
        let samples = vec![threshold_sample - 10, -(threshold_sample - 10)];
        assert!(peak_amplitude(&samples) < config.threshold_fraction);
    }

    // ── PreRollBuffer ─────────────────────────────────────────────────────────

    #[test]
    fn test_pre_roll_bounded_by_capacity() {
        let mut buf = PreRollBuffer {
            buf: VecDeque::new(),
            capacity: 5,
        };
        buf.push_batch(&[1, 2, 3]);
        buf.push_batch(&[4, 5, 6, 7]);
        assert_eq!(buf.buf.len(), 5);
    }

    #[test]
    fn test_pre_roll_drain_clears_buffer() {
        let mut buf = PreRollBuffer {
            buf: VecDeque::new(),
            capacity: 10,
        };
        buf.push_batch(&[1, 2, 3]);
        let drained = buf.drain();
        assert_eq!(drained, vec![1, 2, 3]);
        assert!(buf.buf.is_empty());
    }

    #[test]
    fn test_pre_roll_keeps_most_recent_samples() {
        let mut buf = PreRollBuffer {
            buf: VecDeque::new(),
            capacity: 3,
        };
        buf.push_batch(&[1, 2, 3, 4, 5]);
        let drained = buf.drain();
        assert_eq!(drained, vec![3, 4, 5]);
    }

    // ── finalize_or_discard ───────────────────────────────────────────────────

    #[test]
    fn test_finalize_discards_segment_below_minimum_duration() {
        let dir = make_temp_dir();
        let (tx, rx) = mpsc::channel();
        let config = MonitorConfig::default();

        let seg = SoundSegment {
            samples: vec![1000i16; 10],
            sample_rate: 44100,
            channels: 1,
            start_timestamp: "recording_discard_test.wav".to_string(),
        };

        finalize_or_discard(seg, &config, &dir, &tx);

        let first = rx.recv().unwrap();
        assert!(matches!(first, MonitorEvent::SegmentDiscarded { .. }));

        let wav_count = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|x| x.eq_ignore_ascii_case("wav"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(wav_count, 0, "no wav file should be created for a short segment");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_finalize_saves_segment_meeting_minimum_duration() {
        let dir = make_temp_dir();
        let (tx, rx) = mpsc::channel();
        let config = MonitorConfig::default();

        let sample_rate: u32 = 44100;
        let min_samples =
            (sample_rate as f64 * config.min_clip_duration.as_secs_f64()).ceil() as usize + 1000;

        let seg = SoundSegment {
            samples: vec![1000i16; min_samples],
            sample_rate,
            channels: 1,
            start_timestamp: "recording_save_test.wav".to_string(),
        };

        finalize_or_discard(seg, &config, &dir, &tx);

        let first = rx.recv().unwrap();
        assert!(
            matches!(first, MonitorEvent::SegmentSaved(_)),
            "expected SegmentSaved"
        );

        if let MonitorEvent::SegmentSaved(path) = first {
            assert!(path.exists(), "saved file must exist on disk");
        }

        let _ = fs::remove_dir_all(&dir);
    }

    // ── profile dispatch ─────────────────────────────────────────────────────

    fn finalize_segment_with_profile(
        profile: crate::model::AudioOutputProfile,
        filename: &str,
    ) -> hound::WavSpec {
        let dir = make_temp_dir();
        let (tx, _rx) = mpsc::channel();
        let mut config = MonitorConfig::default();
        config.output_profile = profile;

        let sample_rate: u32 = 44100;
        let min_samples =
            (sample_rate as f64 * config.min_clip_duration.as_secs_f64()).ceil() as usize + 1000;

        let seg = SoundSegment {
            samples: vec![1000i16; min_samples],
            sample_rate,
            channels: 1,
            start_timestamp: filename.to_string(),
        };

        finalize_or_discard(seg, &config, &dir, &tx);

        let path = dir.join(filename);
        let spec = hound::WavReader::open(&path).unwrap().spec();
        let _ = fs::remove_dir_all(&dir);
        spec
    }

    #[test]
    fn test_finalize_honors_pcm8_profile() {
        use crate::model::{AudioOutputProfile, CompressionProfile, SupportedFormat};
        let spec = finalize_segment_with_profile(
            AudioOutputProfile {
                format: SupportedFormat::Wav,
                compression: CompressionProfile::Pcm8,
            },
            "monitor_pcm8.wav",
        );
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);
        assert_eq!(spec.bits_per_sample, 8);
    }

    #[test]
    fn test_finalize_honors_pcm24_profile() {
        use crate::model::{AudioOutputProfile, CompressionProfile, SupportedFormat};
        let spec = finalize_segment_with_profile(
            AudioOutputProfile {
                format: SupportedFormat::Wav,
                compression: CompressionProfile::Pcm24,
            },
            "monitor_pcm24.wav",
        );
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);
        assert_eq!(spec.bits_per_sample, 24);
    }

    #[test]
    fn test_finalize_honors_float32_profile() {
        use crate::model::{AudioOutputProfile, CompressionProfile, SupportedFormat};
        let spec = finalize_segment_with_profile(
            AudioOutputProfile {
                format: SupportedFormat::Wav,
                compression: CompressionProfile::Float32,
            },
            "monitor_float32.wav",
        );
        assert_eq!(spec.sample_format, hound::SampleFormat::Float);
        assert_eq!(spec.bits_per_sample, 32);
    }

    #[test]
    fn test_finalize_defaults_to_pcm16() {
        let spec = finalize_segment_with_profile(
            crate::model::AudioOutputProfile::default(),
            "monitor_default.wav",
        );
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);
        assert_eq!(spec.bits_per_sample, 16);
    }
}