//! Helpers shared by every CLI subcommand handler.
//!
//! These utilities live here so each command file (`init`, `create`,
//! `get`, `list`, `tree`, `query`, `validate`) can stay focused on its
//! own logic without re-implementing serialization, store opening, or
//! input reading.

use std::env;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::OutputFormat;
use crate::store::FileStore;

/// Serializes `value` to stdout in the requested [`OutputFormat`].
pub fn output<T: Serialize>(value: &T, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Yaml => {
            print!("{}", serde_yaml_ng::to_string(value)?);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
    }
    Ok(())
}

/// Reads input from `path`, treating `-` as stdin.
pub fn read_from(path: &str) -> Result<String> {
    if path == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        Ok(std::fs::read_to_string(path)?)
    }
}

/// Opens the [`FileStore`] anchored at the current working directory.
pub fn open_store() -> Result<FileStore> {
    FileStore::open(&env::current_dir()?)
}

/// Helper used by every `create` branch to print the result of a
/// successful create.
pub fn report_created<T: Serialize>(
    id_only: bool,
    id: &str,
    path: &Path,
    node: &T,
    format: OutputFormat,
) -> Result<()> {
    if id_only {
        println!("{id}");
    } else {
        eprintln!("Created {}", path.display());
        output(node, format)?;
    }
    Ok(())
}
