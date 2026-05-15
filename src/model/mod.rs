use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct WavFileEntry {
    pub name: String,
    pub path: PathBuf,
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

pub struct TuiContext {
    pub selected_index: Option<usize>,
    pub wav_files: Vec<WavFileEntry>,
    pub status_message: Option<String>,
    pub app_state: AppState,
}

impl TuiContext {
    pub fn new() -> Self {
        Self {
            selected_index: None,
            wav_files: Vec::new(),
            status_message: None,
            app_state: AppState::Idle,
        }
    }
}
