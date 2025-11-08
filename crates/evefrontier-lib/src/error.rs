use std::path::PathBuf;

use thiserror::Error;

/// Convenient result alias for the EveFrontier library.
pub type Result<T> = std::result::Result<T, Error>;

/// Top-level library error type.
#[derive(Debug, Error)]
pub enum Error {
    /// Dataset could not be located at the resolved path.
    #[error("dataset not found at {path}")]
    DatasetNotFound { path: PathBuf },

    /// No suitable project directories could be resolved for this platform.
    #[error("failed to resolve project directories for dataset cache")]
    ProjectDirsUnavailable,

    /// The downloader has not been implemented yet.
    #[error("dataset download is not yet implemented")]
    DownloadNotImplemented,

    /// Raised when attempting to load a schema that is not supported.
    #[error("unsupported dataset schema; expected SolarSystems/Jumps or mapSolarSystems tables")]
    UnsupportedSchema,

    /// Raised when a system name could not be found in the dataset.
    #[error("unknown system name: {name}")]
    UnknownSystem { name: String },

    /// Raised when no route could be found between two systems.
    #[error("no route found between {start} and {goal}")]
    RouteNotFound { start: String, goal: String },

    /// Wrapper for SQLite errors.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    /// Wrapper for IO errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
