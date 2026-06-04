use thiserror::Error;

pub type PrunifierResult<T> = Result<T, PrunifierError>;

#[derive(Error, Debug)]
pub enum PrunifierError {
    #[error("Scheme not found: {0}")]
    SchemeNotFound(String),

    #[error("Invalid scheme: {0}")]
    InvalidScheme(String),

    #[error("Command failed: {0} (exit code: {1})")]
    CommandFailed(String, i32),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Recursion detected: prunify cannot proxy itself")]
    RecursionDetected,
}
