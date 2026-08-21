//! Error types for sorocheck.

use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur during checking.
#[derive(Debug, Error)]
pub enum Error {
    /// File I/O error (e.g., can't read a file)
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Parse error (e.g., malformed Rust code)
    #[error("failed to parse {path}: {message}")]
    Parse { path: PathBuf, message: String },

    /// Other miscellaneous errors
    #[error("{0}")]
    Other(String),
}