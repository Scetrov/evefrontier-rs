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

    /// No suitable cache directory could be resolved for storing download artifacts.
    #[error("failed to resolve cache directories for dataset downloads")]
    CacheDirsUnavailable,

    /// Raised when the GitHub release did not contain a usable dataset asset.
    #[error("no dataset asset found in GitHub release {tag}")]
    DatasetAssetMissing { tag: String },

    /// Raised when a specific dataset release tag does not exist.
    #[error("dataset release {tag} not found on GitHub")]
    DatasetReleaseNotFound { tag: String },

    /// Raised when an archive asset did not contain a `.db` file.
    #[error("archive {archive} did not contain a dataset database file")]
    ArchiveMissingDatabase { archive: PathBuf },

    /// Raised when attempting to load a schema that is not supported.
    #[error("unsupported dataset schema; expected SolarSystems/Jumps or mapSolarSystems tables")]
    UnsupportedSchema,

    /// Raised when a system name could not be found in the dataset.
    #[error("unknown system name: {name}{}", format_suggestions(.suggestions))]
    UnknownSystem {
        name: String,
        suggestions: Vec<String>,
    },

    /// Raised when no route could be found between two systems.
    #[error("no route found between {start} and {goal}")]
    RouteNotFound { start: String, goal: String },

    /// Raised when a computed route plan lacks any systems.
    #[error("route plan was empty")]
    EmptyRoutePlan,

    /// Raised when a requested routing option is not yet supported.
    #[error("route option {option} is not supported yet")]
    UnsupportedRouteOption { option: String },

    /// Wrapper for SQLite errors.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    /// Wrapper for IO errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Wrapper for HTTP client errors.
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    /// Wrapper for ZIP archive parsing errors.
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
}

fn format_suggestions(suggestions: &[String]) -> String {
    if suggestions.is_empty() {
        String::new()
    } else if suggestions.len() == 1 {
        format!(". Did you mean '{}'?", suggestions[0])
    } else {
        format!(
            ". Did you mean one of: {}?",
            suggestions
                .iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
