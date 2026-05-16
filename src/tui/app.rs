use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use ratatui::backend::Backend;

use crate::audio;
use crate::error::AppError;
use crate::model::{
    AppState, MonitorEvent, MonitoringHandle, MonitoringSubState, PlaybackHandle, RecordingHandle,
    TuiContext, WavFileEntry,
};

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
                    KeyCode::Char('m') => handle_monitor(ctx)?,
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
    // ── Recording completion ──────────────────────────────────────────────────
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

    // ── Playback completion ───────────────────────────────────────────────────
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

    // ── Monitoring events ─────────────────────────────────────────────────────
    let (monitoring_events, monitoring_disconnected) =
        if let AppState::Monitoring(h) = &ctx.app_state {
            let mut events = Vec::new();
            let mut disconnected = false;
            loop {
                match h.event_rx.try_recv() {
                    Ok(e) => events.push(e),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
            (events, disconnected)
        } else {
            (Vec::new(), false)
        };

    // ── Dispatch recording result ─────────────────────────────────────────────
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
        return;
    }

    // ── Dispatch playback result ──────────────────────────────────────────────
    if let Some(result) = playing_recv {
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
        return;
    }

    // ── Dispatch monitoring events ────────────────────────────────────────────
    for event in monitoring_events {
        match event {
            MonitorEvent::SubStateChanged(sub) => {
                if let AppState::Monitoring(h) = &mut ctx.app_state {
                    h.sub_state = sub;
                }
            }
            MonitorEvent::SegmentSaved(path) => {
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
            MonitorEvent::SegmentDiscarded { reason } => {
                if let AppState::Monitoring(h) = &mut ctx.app_state {
                    h.sub_state = MonitoringSubState::Listening;
                }
                ctx.status_message = Some(format!("Discarded: {reason}"));
            }
            MonitorEvent::ContinuousTriggering => {
                ctx.status_message = Some(
                    "Warning: threshold may be too low — continuous triggering detected"
                        .to_string(),
                );
            }
            MonitorEvent::Failed(e) => {
                if let AppState::Monitoring(handle) =
                    std::mem::replace(&mut ctx.app_state, AppState::Idle)
                {
                    let _ = handle.thread.join();
                }
                ctx.status_message = Some(format!("Monitor error: {e}"));
                return;
            }
        }
    }

    if monitoring_disconnected {
        if let AppState::Monitoring(handle) =
            std::mem::replace(&mut ctx.app_state, AppState::Idle)
        {
            let _ = handle.thread.join();
        }
        // Clear "Stopping…" once the thread exits cleanly; preserve Saved/Discarded messages.
        if ctx.status_message.as_deref() == Some("Stopping…") {
            ctx.status_message = None;
        }
    }
}

fn handle_record(ctx: &mut TuiContext) -> Result<(), AppError> {
    if !matches!(ctx.app_state, AppState::Idle) {
        return Ok(());
    }

    let Some(defaults) = ctx.defaults.as_ref() else {
        ctx.status_message = Some(
            "Audio defaults invalid — fix config/audio.conf".to_string(),
        );
        return Ok(());
    };
    let profile = defaults.profile;

    let recordings_dir = PathBuf::from("recordings");
    audio::record::ensure_recordings_dir(&recordings_dir)?;

    let stop_flag = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();
    let thread =
        audio::record::start_recording_thread(Arc::clone(&stop_flag), tx, recordings_dir, profile);

    ctx.app_state = AppState::Recording(RecordingHandle {
        stop_flag,
        result_rx: rx,
        thread,
    });
    ctx.status_message = Some("Recording… press 's' to stop".to_string());
    Ok(())
}

fn handle_monitor(ctx: &mut TuiContext) -> Result<(), AppError> {
    if matches!(ctx.app_state, AppState::Playing(_)) {
        ctx.status_message = Some("Stop playback before monitoring".to_string());
        return Ok(());
    }
    if !matches!(ctx.app_state, AppState::Idle) {
        return Ok(());
    }

    let Some(defaults) = ctx.defaults.as_ref() else {
        ctx.status_message = Some(
            "Audio defaults invalid — fix config/audio.conf".to_string(),
        );
        return Ok(());
    };
    let profile = defaults.profile;

    let recordings_dir = PathBuf::from("recordings");
    audio::record::ensure_recordings_dir(&recordings_dir)?;

    let stop_flag = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();
    let mut config = audio::monitor::MonitorConfig::default();
    config.output_profile = profile;
    let thread = audio::monitor::start_monitoring_thread(
        Arc::clone(&stop_flag),
        tx,
        recordings_dir,
        config,
    );

    ctx.app_state = AppState::Monitoring(MonitoringHandle {
        stop_flag,
        event_rx: rx,
        thread,
        sub_state: MonitoringSubState::Listening,
    });
    ctx.status_message = Some("Monitoring — press 's' to stop".to_string());
    Ok(())
}

fn handle_play(ctx: &mut TuiContext) -> Result<(), AppError> {
    if matches!(ctx.app_state, AppState::Monitoring(_)) {
        ctx.status_message = Some("Stop monitoring before playback".to_string());
        return Ok(());
    }
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
        AppState::Monitoring(h) => {
            h.stop_flag.store(true, Ordering::Relaxed);
            ctx.status_message = Some("Stopping…".to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MonitorEvent, MonitoringSubState, PlaybackHandle};
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    fn make_monitoring_ctx() -> (TuiContext, mpsc::Sender<MonitorEvent>) {
        let (tx, rx) = mpsc::channel::<MonitorEvent>();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread = std::thread::spawn(|| {});
        let mut ctx = TuiContext::new();
        ctx.app_state = AppState::Monitoring(MonitoringHandle {
            stop_flag,
            event_rx: rx,
            thread,
            sub_state: MonitoringSubState::Listening,
        });
        (ctx, tx)
    }

    #[test]
    fn test_handle_monitor_while_playing_shows_rejection() {
        let (tx, rx) = mpsc::channel::<Result<(), AppError>>();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread = std::thread::spawn(|| {});
        let mut ctx = TuiContext::new();
        ctx.app_state = AppState::Playing(PlaybackHandle {
            stop_flag,
            result_rx: rx,
            thread,
            source_path: PathBuf::from("dummy.wav"),
        });
        drop(tx);

        handle_monitor(&mut ctx).unwrap();

        assert!(
            matches!(ctx.app_state, AppState::Playing(_)),
            "app_state should remain Playing"
        );
        assert_eq!(
            ctx.status_message.as_deref(),
            Some("Stop playback before monitoring"),
        );
    }

    #[test]
    fn test_segment_saved_event_sets_status_message() {
        let (mut ctx, tx) = make_monitoring_ctx();
        let saved_path = PathBuf::from("recordings/recording_test.wav");
        tx.send(MonitorEvent::SegmentSaved(saved_path.clone())).unwrap();
        drop(tx);

        check_audio_completion(&mut ctx);

        assert!(
            ctx.status_message
                .as_deref()
                .unwrap_or("")
                .starts_with("Saved:"),
            "status should start with 'Saved:' after SegmentSaved event"
        );
    }

    #[test]
    fn test_segment_discarded_event_sets_status_message() {
        let (mut ctx, tx) = make_monitoring_ctx();
        tx.send(MonitorEvent::SegmentDiscarded {
            reason: "segment too short (120ms)".to_string(),
        })
        .unwrap();
        drop(tx);

        check_audio_completion(&mut ctx);

        assert_eq!(
            ctx.status_message.as_deref(),
            Some("Discarded: segment too short (120ms)"),
        );
    }

    #[test]
    fn test_continuous_triggering_shows_warning() {
        let (mut ctx, tx) = make_monitoring_ctx();
        tx.send(MonitorEvent::ContinuousTriggering).unwrap();
        drop(tx);

        check_audio_completion(&mut ctx);

        assert!(
            ctx.status_message
                .as_deref()
                .unwrap_or("")
                .contains("threshold may be too low"),
        );
    }

    #[test]
    fn test_thread_disconnect_transitions_to_idle() {
        let (mut ctx, tx) = make_monitoring_ctx();
        ctx.status_message = Some("Stopping…".to_string());
        drop(tx);

        check_audio_completion(&mut ctx);

        assert!(matches!(ctx.app_state, AppState::Idle));
        assert_eq!(ctx.status_message, None, "Stopping… should be cleared on clean exit");
    }

    // ── US3: defaults gate ────────────────────────────────────────────────────

    #[test]
    fn test_handle_record_is_gated_when_defaults_missing() {
        let mut ctx = TuiContext::new();
        assert!(ctx.defaults.is_none());

        handle_record(&mut ctx).unwrap();

        assert!(matches!(ctx.app_state, AppState::Idle));
        assert_eq!(
            ctx.status_message.as_deref(),
            Some("Audio defaults invalid — fix config/audio.conf"),
        );
    }

    #[test]
    fn test_handle_monitor_is_gated_when_defaults_missing() {
        let mut ctx = TuiContext::new();
        assert!(ctx.defaults.is_none());

        handle_monitor(&mut ctx).unwrap();

        assert!(matches!(ctx.app_state, AppState::Idle));
        assert_eq!(
            ctx.status_message.as_deref(),
            Some("Audio defaults invalid — fix config/audio.conf"),
        );
    }
}
