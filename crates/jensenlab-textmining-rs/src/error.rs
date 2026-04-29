//! Error types for `JensenLab` text mining data parsing.

use thiserror::Error;

/// Errors that can occur when parsing `JensenLab` text mining files.
#[derive(Error, Debug)]
pub enum Error {
    /// An error from the CSV parser.
    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),

    /// An error from the Arrow library.
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    /// An error from the gzip decompressor.
    #[error("Gzip decompression error: {0}")]
    Gzip(#[from] std::io::Error),

    /// An invalid entity type code was encountered.
    #[error("Invalid entity type code: {0}")]
    InvalidEntityType(i64),

    /// A timestamp parsing error.
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    /// A confidence score parsing error.
    #[error("Invalid confidence score: {0}")]
    InvalidScore(String),

    /// A required column is missing from the input.
    #[error("Missing required column: {0}")]
    MissingColumn(String),
}

/// Result type alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;
