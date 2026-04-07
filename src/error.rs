//! Domain error types and the top-level [`CliError`] wrapper that maps
//! them onto process exit codes.
//!
//! The CLI uses a layered error strategy: each subsystem owns a typed
//! error ([`StoreError`], [`CreateError`], [`ValidationError`]) and the
//! binary entry point converts whatever bubbles up into a single
//! [`CliError`] whose [`CliError::exit_code`] decides how the process
//! terminates. See constraint 14 (Domain Error Types) in `.shapes/`.

use std::process::ExitCode;

use thiserror::Error;

use crate::model::NodeType;

/// Errors raised by the file-backed [`crate::store`] layer.
//
// `dead_code` is allowed because some variants are introduced
// incrementally as `store.rs` migrates from `anyhow`. Remove this
// allow once every variant is constructed somewhere.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum StoreError {
    /// A node of `node_type` with the given `id` could not be located.
    #[error("{node_type} {id} not found")]
    NotFound {
        /// Type of node that was missing.
        node_type: NodeType,
        /// Raw identifier the caller asked for.
        id: u64,
    },

    /// A node already exists at the given filesystem path.
    #[error("{node_type} already exists in {path}")]
    AlreadyExists {
        /// Human-readable name of the node type.
        node_type: String,
        /// Path on disk that already holds the node.
        path: String,
    },

    /// An I/O failure occurred while reading or writing a file.
    #[error("failed to read {path}: {source}")]
    Io {
        /// Path that failed.
        path: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// A YAML parse failure while loading a node.
    #[error("failed to parse {path}: {source}")]
    Parse {
        /// Path that failed to parse.
        path: String,
        /// Underlying serde error.
        source: serde_yml::Error,
    },

    /// `.shapes/` directory is missing — the user has not run
    /// `shapes init`.
    #[error(".shapes/ directory not found — run `shapes init` first")]
    NotInitialized,

    /// `.shapes/` directory already exists at init time.
    #[error(".shapes/ directory already exists")]
    AlreadyInitialized,
}

/// Errors raised when creating a new node.
//
// `dead_code` is allowed because some variants are wired in by the
// profile-aware create flow added incrementally. Remove this allow
// once `--profile` is fully wired in every command branch.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum CreateError {
    /// `--profile <id>` referenced a profile that does not exist.
    #[error("profile {id} not found — cannot apply --profile")]
    ProfileNotFound {
        /// Raw profile identifier supplied by the caller.
        id: u64,
    },

    /// The requested `kind` is not declared by the profile's allowed
    /// kinds list.
    #[error("kind '{kind}' not allowed by profile {profile_id} — allowed: {allowed}")]
    InvalidKind {
        /// The disallowed kind value.
        kind: String,
        /// Profile that rejected the kind.
        profile_id: u64,
        /// Comma-separated list of accepted kinds for context.
        allowed: String,
    },

    /// A store-layer failure surfaced through the create flow.
    #[error("{0}")]
    Store(#[from] StoreError),
}

/// Errors raised by the `validate` subcommand.
///
/// Modeled as a typed error so the CLI can return the dedicated exit
/// code 2 instead of conflating "validation issues found" with "the
/// program crashed".
#[derive(Debug, Error)]
pub enum ValidationError {
    /// One or more validation issues were reported during a run.
    #[error("{count} validation issue(s) found")]
    IssuesFound {
        /// Number of issues that were emitted.
        count: usize,
    },
}

/// Top-level error wrapper used by `main` to compute exit codes and
/// format messages.
#[derive(Debug, Error)]
pub enum CliError {
    /// Store-layer failure.
    #[error("{0}")]
    Store(#[from] StoreError),

    /// Create-flow failure.
    #[error("{0}")]
    Create(#[from] CreateError),

    /// Validation-flow failure.
    #[error("{0}")]
    Validation(#[from] ValidationError),

    /// Anything else, including `anyhow` errors from third-party crates
    /// that have not yet been migrated to a domain type.
    #[error("{0:#}")]
    Other(#[from] anyhow::Error),
}

impl CliError {
    /// Returns the process exit code that should be reported for this
    /// error: `2` for validation failures, `1` for everything else.
    #[must_use]
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::Validation(ValidationError::IssuesFound { .. }) => ExitCode::from(2),
            _ => ExitCode::from(1),
        }
    }
}
