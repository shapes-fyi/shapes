//! Domain error types and the top-level [`CliError`] wrapper that maps
//! them onto process exit codes.
//!
//! The CLI uses a layered error strategy: each subsystem owns a typed
//! error ([`CreateError`], [`ValidationError`]) and the binary entry
//! point converts whatever bubbles up into a single [`CliError`] whose
//! [`CliError::exit_code`] decides how the process terminates. See
//! constraint 14 (Domain Error Types) in `.shapes/`.

use std::process::ExitCode;

use thiserror::Error;

/// Errors raised when creating a new node.
#[derive(Debug, Error)]
pub enum CreateError {
    /// The requested `kind` is not declared by the governing profile's
    /// allow-list.
    #[error("kind '{kind}' not allowed by profile {profile_id} — allowed: {allowed}")]
    InvalidKind {
        /// The disallowed kind value.
        kind: String,
        /// Profile that rejected the kind.
        profile_id: u64,
        /// Comma-separated list of accepted kinds for context.
        allowed: String,
    },
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

/// Errors raised by the `ci-check` subcommand.
///
/// Modeled as a typed error so the CLI can return the dedicated exit
/// code 2 (matching `validate`) when PR-level checks fail, distinct
/// from a generic crash.
#[derive(Debug, Error)]
pub enum CiCheckError {
    /// One or more PR-level issues were reported during a run.
    #[error("{count} ci-check issue(s) found")]
    IssuesFound {
        /// Number of issues that were emitted.
        count: usize,
    },
}

/// Top-level error wrapper used by `main` to compute exit codes and
/// format messages.
#[derive(Debug, Error)]
pub enum CliError {
    /// Create-flow failure.
    #[error("{0}")]
    Create(#[from] CreateError),

    /// Validation-flow failure.
    #[error("{0}")]
    Validation(#[from] ValidationError),

    /// CI-check-flow failure.
    #[error("{0}")]
    CiCheck(#[from] CiCheckError),

    /// Anything else the store or command layer raised through
    /// `anyhow`.
    #[error("{0:#}")]
    Other(#[from] anyhow::Error),
}

impl CliError {
    /// Returns the process exit code that should be reported for this
    /// error: `2` for validation or ci-check failures, `1` for
    /// everything else.
    #[must_use]
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::Validation(ValidationError::IssuesFound { .. })
            | CliError::CiCheck(CiCheckError::IssuesFound { .. }) => ExitCode::from(2),
            _ => ExitCode::from(1),
        }
    }
}
