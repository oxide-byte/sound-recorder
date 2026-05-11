pub mod app;
pub mod view;

use crate::error::AppError;

pub fn run_tui() -> Result<(), AppError> {
    app::run()
}