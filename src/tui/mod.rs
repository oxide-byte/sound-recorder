pub mod app;
pub mod view;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::panic;

use crate::error::AppError;
use crate::model::TuiContext;

pub fn run_tui() -> Result<(), AppError> {
    // Ensure terminal is restored if a panic occurs
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        default_hook(info);
    }));

    enable_raw_mode().map_err(AppError::Io)?;
    io::stdout()
        .execute(EnterAlternateScreen)
        .map_err(AppError::Io)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).map_err(AppError::Io)?;

    let mut ctx = TuiContext::new();
    let recordings_dir = std::path::Path::new("recordings");
    ctx.wav_files = app::scan_wav_files(recordings_dir);
    if !ctx.wav_files.is_empty() {
        ctx.selected_index = Some(0);
    }

    let result = app::run_event_loop(&mut terminal, &mut ctx);

    disable_raw_mode().map_err(AppError::Io)?;
    io::stdout()
        .execute(LeaveAlternateScreen)
        .map_err(AppError::Io)?;

    result
}
