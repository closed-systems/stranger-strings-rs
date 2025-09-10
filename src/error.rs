use thiserror::Error;

/// Errors that can occur during string analysis
#[derive(Error, Debug)]
pub enum StrangerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Model parsing error: {0}")]
    ModelParsing(String),

    #[error("Model not loaded - call load_model() first")]
    ModelNotLoaded,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StrangerError>;