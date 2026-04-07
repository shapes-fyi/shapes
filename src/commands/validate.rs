//! `shapes validate` — runs graph integrity checks and emits issues
//! using the requested output format.

use crate::OutputFormat;
use crate::commands::dag;
use crate::commands::shared::open_store;
use crate::error::{CliError, ValidationError};

/// Validates the graph and reports issues. Exit code is set by the
/// [`CliError::exit_code`](crate::error::CliError::exit_code) mapping
/// for [`ValidationError::IssuesFound`].
pub fn validate(format: OutputFormat) -> Result<(), CliError> {
    let store = open_store()?;
    let issues = dag::validate(&store)?;
    if issues.is_empty() {
        match format {
            OutputFormat::Json => println!("[]"),
            OutputFormat::Yaml => eprintln!("No issues found."),
        }
        Ok(())
    } else {
        let count = issues.len();
        match format {
            OutputFormat::Json => {
                let json =
                    serde_json::to_string_pretty(&issues).map_err(|e| CliError::Other(e.into()))?;
                println!("{json}");
            }
            OutputFormat::Yaml => {
                for issue in &issues {
                    eprintln!("{issue}");
                }
                eprintln!("{count} validation issue(s) found");
            }
        }
        Err(ValidationError::IssuesFound { count }.into())
    }
}
