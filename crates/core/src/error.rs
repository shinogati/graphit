use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphitError {
    #[error("Graphit '{0}' not found")]
    NotFound(String),

    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}