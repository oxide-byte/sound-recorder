mod audio;
mod config;
mod error;
mod model;
mod tui;

fn main() {
    if let Err(err) = tui::run_tui() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}