//! Error types for jetti.
//!
//! All fallible functions in jetti return [`Result<T>`], which uses [`JettiError`]
//! as the error type.

use std::path::PathBuf;

use thiserror::Error;

/// Convenience alias for `Result<T, JettiError>`.
pub type Result<T> = std::result::Result<T, JettiError>;

#[derive(Debug, Error)]
pub enum JettiError {
    /// Filesystem I/O failed (read, write, remove, create_dir, walk).
    #[error("failed to {action} {}: {source}", path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },

    /// Could not parse a repository specifier.
    #[error("{0}")]
    InvalidRepo(String),

    /// A subprocess (git, $EDITOR) failed to launch or exited non-zero.
    #[error("{0}")]
    Subprocess(String),

    /// An environment precondition is not met (git not installed, $EDITOR not set).
    #[error("{0}")]
    Precondition(String),

    /// Rayon thread pool creation failed.
    #[error("failed to create thread pool: {0}")]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),
}
