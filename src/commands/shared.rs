//! Helpers shared by every CLI subcommand handler.
//!
//! These utilities live here so each command file (`init`, `create`,
//! `get`, `list`, `tree`, `query`, `validate`) can stay focused on its
//! own logic without re-implementing serialization, store opening, or
//! input reading.

use std::env;
use std::io::Read;
use std::path::Path;

use anyhow::{Result, bail};
use serde::Serialize;

use crate::OutputFormat;
use crate::store::{CURRENT_STORE_VERSION, FileStore};

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

/// Opens the [`FileStore`] anchored at the current working directory
/// and enforces the schema-version gate.
///
/// Stores whose `meta.yaml` version does not match
/// [`CURRENT_STORE_VERSION`] are rejected with an actionable error
/// pointing at `shapes migrate`. This catches outdated stores before
/// any typed deserialization runs, so users never see cryptic serde
/// errors for schemas the current CLI no longer understands.
pub fn open_store() -> Result<FileStore> {
    let store = FileStore::open(&env::current_dir()?)?;
    let meta = store.read_meta()?;
    if meta.version != CURRENT_STORE_VERSION {
        bail!(
            ".shapes/ store is at version {} but this CLI expects {}. \
             Run `shapes migrate` to upgrade.",
            meta.version,
            CURRENT_STORE_VERSION,
        );
    }
    Ok(store)
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
