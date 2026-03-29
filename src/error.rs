use std::process::ExitCode;

use thiserror::Error;

use crate::model::NodeType;

// ---------------------------------------------------------------------------
// StoreError — file-system and persistence operations
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
#[allow(dead_code)] // Variants used incrementally as store.rs migrates from anyhow
pub enum StoreError {
    #[error("{node_type} {id} not found")]
    NotFound { node_type: NodeType, id: u64 },

    #[error("{node_type} already exists in {path}")]
    AlreadyExists { node_type: String, path: String },

    #[error("failed to read {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },

    #[error("failed to parse {path}: {source}")]
    Parse {
        path: String,
        source: serde_yml::Error,
    },

    #[error(".shapes/ directory not found — run `shapes init` first")]
    NotInitialized,

    #[error(".shapes/ directory already exists")]
    AlreadyInitialized,
}

// ---------------------------------------------------------------------------
// CreateError — node creation failures
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
#[allow(dead_code)] // Variants used when --profile flag is added in PR 6
pub enum CreateError {
    #[error("profile {id} not found — cannot apply --profile")]
    ProfileNotFound { id: u64 },

    #[error("kind '{kind}' not allowed by profile {profile_id} — allowed: {allowed}")]
    InvalidKind {
        kind: String,
        profile_id: u64,
        allowed: String,
    },

    #[error("{0}")]
    Store(#[from] StoreError),
}

// ---------------------------------------------------------------------------
// ValidationError — validation-as-operation control flow
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("{count} validation issue(s) found")]
    IssuesFound { count: usize },
}

// ---------------------------------------------------------------------------
// CliError — top-level wrapper controlling exit codes
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Store(#[from] StoreError),

    #[error("{0}")]
    Create(#[from] CreateError),

    #[error("{0}")]
    Validation(#[from] ValidationError),

    #[error("{0:#}")]
    Other(#[from] anyhow::Error),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::Validation(ValidationError::IssuesFound { .. }) => ExitCode::from(2),
            _ => ExitCode::from(1),
        }
    }
}
