use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedFormat {
    Wav,
}

impl SupportedFormat {
    pub fn from_id(s: &str) -> Result<Self, AppError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "wav" => Ok(SupportedFormat::Wav),
            other => Err(AppError::Config(format!(
                "unsupported format '{other}'; supported: wav"
            ))),
        }
    }

    pub fn as_id(&self) -> &'static str {
        match self {
            SupportedFormat::Wav => "wav",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionProfile {
    Pcm8,
    Pcm16,
    Pcm24,
    Float32,
}

impl CompressionProfile {
    pub fn from_id(s: &str) -> Result<Self, AppError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "pcm8" => Ok(CompressionProfile::Pcm8),
            "pcm16" => Ok(CompressionProfile::Pcm16),
            "pcm24" => Ok(CompressionProfile::Pcm24),
            "float32" => Ok(CompressionProfile::Float32),
            other => Err(AppError::Config(format!(
                "unsupported compression '{other}'; supported: pcm8, pcm16, pcm24, float32"
            ))),
        }
    }

    pub fn as_id(&self) -> &'static str {
        match self {
            CompressionProfile::Pcm8 => "pcm8",
            CompressionProfile::Pcm16 => "pcm16",
            CompressionProfile::Pcm24 => "pcm24",
            CompressionProfile::Float32 => "float32",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioOutputProfile {
    pub format: SupportedFormat,
    pub compression: CompressionProfile,
}

impl AudioOutputProfile {
    pub fn validated(
        format: SupportedFormat,
        compression: CompressionProfile,
    ) -> Result<Self, AppError> {
        match (format, compression) {
            (SupportedFormat::Wav, _) => Ok(Self { format, compression }),
        }
    }
}

impl Default for AudioOutputProfile {
    fn default() -> Self {
        Self {
            format: SupportedFormat::Wav,
            compression: CompressionProfile::Pcm16,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WavFileEntry {
    pub name: String,
    pub path: PathBuf,
    pub created_at: String,
}

pub struct RecordingHandle {
    pub stop_flag: Arc<AtomicBool>,
    pub result_rx: mpsc::Receiver<Result<PathBuf, AppError>>,
    pub thread: JoinHandle<()>,
}

pub struct PlaybackHandle {
    pub stop_flag: Arc<AtomicBool>,
    pub result_rx: mpsc::Receiver<Result<(), AppError>>,
    pub thread: JoinHandle<()>,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MonitoringSubState {
    Listening,
    Capturing,
}

pub enum MonitorEvent {
    SubStateChanged(MonitoringSubState),
    SegmentSaved(PathBuf),
    SegmentDiscarded { reason: String },
    ContinuousTriggering,
    Failed(AppError),
}

pub struct MonitoringHandle {
    pub stop_flag: Arc<AtomicBool>,
    pub event_rx: mpsc::Receiver<MonitorEvent>,
    pub thread: JoinHandle<()>,
    pub sub_state: MonitoringSubState,
}

pub enum AppState {
    Idle,
    Recording(RecordingHandle),
    Playing(PlaybackHandle),
    Monitoring(MonitoringHandle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    Single,
    Continuous,
    Loop,
}

impl PlayMode {
    pub fn next(&self) -> Self {
        match self {
            PlayMode::Single => PlayMode::Continuous,
            PlayMode::Continuous => PlayMode::Loop,
            PlayMode::Loop => PlayMode::Single,
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            PlayMode::Single => "[ ]",
            PlayMode::Continuous => "[C]",
            PlayMode::Loop => "[L]",
        }
    }
}

pub struct TuiContext {
    pub selected_index: Option<usize>,
    pub wav_files: Vec<WavFileEntry>,
    pub status_message: Option<String>,
    pub app_state: AppState,
    pub defaults: Option<crate::config::AudioDefaultsConfig>,
    pub play_mode: PlayMode,
}

impl TuiContext {
    pub fn new() -> Self {
        Self {
            selected_index: None,
            wav_files: Vec::new(),
            status_message: None,
            app_state: AppState::Idle,
            defaults: None,
            play_mode: PlayMode::Single,
        }
    }
}