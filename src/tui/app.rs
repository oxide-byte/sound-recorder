use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use ratatui::backend::Backend;

use crate::audio;
use crate::error::AppError;
use crate::model::{AppState, PlaybackHandle, RecordingHandle, TuiContext, WavFileEntry};

pub fn scan_wav_files(dir: &Path) -> Vec<WavFileEntry> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<WavFileEntry> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("wav"))
                .unwrap_or(false)
        })
        .map(|e| WavFileEntry {
            name: e.file_name().to_string_lossy().into_owned(),
            path: e.path(),
        })
        .collect();
    files.sort_by(|a, b| b.name.cmp(&a.name));
    files
}

pub fn run_event_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    ctx: &mut TuiContext,
) -> Result<(), AppError> {
    loop {
        terminal
            .draw(|f| crate::tui::view::render(f, ctx))
            .map_err(AppError::Io)?;

        check_audio_completion(ctx);

        if event::poll(Duration::from_millis(100)).map_err(AppError::Io)? {
            if let Event::Key(key) = event::read().map_err(AppError::Io)? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if matches!(ctx.app_state, AppState::Idle) {
                            break;
                        }
                        ctx.status_message = Some("Stop before quitting".to_string());
                    }
                    KeyCode::Char('r') => handle_record(ctx)?,
                    KeyCode::Char('p') => handle_play(ctx)?,
                    KeyCode::Char('s') => handle_stop(ctx),
                    KeyCode::Up | KeyCode::Char('k') => navigate_up(ctx),
                    KeyCode::Down | KeyCode::Char('j') => navigate_down(ctx),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn check_audio_completion(ctx: &mut TuiContext) {
    let recording_recv: Option<Result<PathBuf, AppError>> =
        if let AppState::Recording(h) = &ctx.app_state {
            match h.result_rx.try_recv() {
                Ok(r) => Some(r),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err(AppError::Audio("recording thread disconnected".into())))
                }
            }
        } else {
            None
        };

    let playing_recv: Option<Result<(), AppError>> =
        if let AppState::Playing(h) = &ctx.app_state {
            match h.result_rx.try_recv() {
                Ok(r) => Some(r),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err(AppError::Audio("playback thread disconnected".into())))
                }
            }
        } else {
            None
        };

    if let Some(result) = recording_recv {
        let AppState::Recording(handle) =
            std::mem::replace(&mut ctx.app_state, AppState::Idle)
        else {
            return;
        };
        let _ = handle.thread.join();
        match result {
            Ok(path) => {
                ctx.wav_files = scan_wav_files(Path::new("recordings"));
                if ctx.selected_index.is_none() && !ctx.wav_files.is_empty() {
                    ctx.selected_index = Some(0);
                }
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                ctx.status_message = Some(format!("Saved: {name}"));
            }
            Err(e) => {
                ctx.status_message = Some(format!("Recording error: {e}"));
            }
        }
    } else if let Some(result) = playing_recv {
        let AppState::Playing(handle) =
            std::mem::replace(&mut ctx.app_state, AppState::Idle)
        else {
            return;
        };
        let _ = handle.thread.join();
        match result {
            Ok(()) => ctx.status_message = None,
            Err(e) => ctx.status_message = Some(format!("Playback error: {e}")),
        }
    }
}

fn handle_record(ctx: &mut TuiContext) -> Result<(), AppError> {
    if !matches!(ctx.app_state, AppState::Idle) {
        return Ok(());
    }

    let recordings_dir = PathBuf::from("recordings");
    audio::record::ensure_recordings_dir(&recordings_dir)?;

    let stop_flag = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();
    let thread = audio::record::start_recording_thread(Arc::clone(&stop_flag), tx, recordings_dir);

    ctx.app_state = AppState::Recording(RecordingHandle {
        stop_flag,
        result_rx: rx,
        thread,
    });
    ctx.status_message = Some("Recording… press 's' to stop".to_string());
    Ok(())
}

fn handle_play(ctx: &mut TuiContext) -> Result<(), AppError> {
    if !matches!(ctx.app_state, AppState::Idle) {
        return Ok(());
    }

    let Some(index) = ctx.selected_index else {
        ctx.status_message = Some("No file selected".to_string());
        return Ok(());
    };

    let Some(entry) = ctx.wav_files.get(index) else {
        ctx.status_message = Some("No file selected".to_string());
        return Ok(());
    };

    let wav_path = entry.path.clone();
    let filename = entry.name.clone();

    let stop_flag = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();
    let thread =
        audio::playback::start_playback_thread(Arc::clone(&stop_flag), tx, wav_path.clone());

    ctx.app_state = AppState::Playing(PlaybackHandle {
        stop_flag,
        result_rx: rx,
        thread,
        source_path: wav_path,
    });
    ctx.status_message = Some(format!("Playing {filename} — press 's' to stop"));
    Ok(())
}

fn handle_stop(ctx: &mut TuiContext) {
    match &ctx.app_state {
        AppState::Recording(h) => {
            h.stop_flag.store(true, Ordering::Relaxed);
            ctx.status_message = Some("Saving…".to_string());
        }
        AppState::Playing(h) => {
            h.stop_flag.store(true, Ordering::Relaxed);
        }
        AppState::Idle => {}
    }
}

fn navigate_up(ctx: &mut TuiContext) {
    if ctx.wav_files.is_empty() {
        return;
    }
    ctx.selected_index = Some(match ctx.selected_index {
        None | Some(0) => 0,
        Some(i) => i - 1,
    });
}

fn navigate_down(ctx: &mut TuiContext) {
    if ctx.wav_files.is_empty() {
        return;
    }
    let last = ctx.wav_files.len() - 1;
    ctx.selected_index = Some(match ctx.selected_index {
        None => 0,
        Some(i) if i >= last => last,
        Some(i) => i + 1,
    });
}
