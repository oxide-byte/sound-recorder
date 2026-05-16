use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(String),
}