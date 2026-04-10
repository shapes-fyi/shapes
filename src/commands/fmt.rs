//! `shapes fmt` — normalize all `.shapes/` YAML files to serde
//! canonical format.
//!
//! Deterministic serialization: the same Rust struct values always
//! produce the same bytes. Running `shapes fmt` ensures every file
//! on disk matches serde's canonical output so that future serde
//! round-trips (e.g. `shapes amendment archive`) produce minimal,
//! field-only diffs.

use std::fs;

use anyhow::Result;
use similar::{ChangeTag, TextDiff};

use crate::model::{Amendment, Constraint, NodeType, Profile, Shape};
use crate::store::NodeStore;

use super::shared::open_store;

/// Normalize all YAML files in `.shapes/` to serde canonical format.
///
/// When `check` is true, reports non-canonical files without writing
/// and returns an error if any are found.
pub fn fmt(check: bool) -> Result<()> {
    let store = open_store()?;
    let mut dirty: u32 = 0;

    for node_type in [
        NodeType::Shape,
        NodeType::Constraint,
        NodeType::Amendment,
        NodeType::Profile,
    ] {
        for id in store.list_ids(node_type)? {
            let path = store.find_file(node_type, id)?;
            let original = fs::read_to_string(&path)?;

            let canonical = match node_type {
                NodeType::Shape => {
                    serde_yaml_ng::to_string(&serde_yaml_ng::from_str::<Shape>(&original)?)?
                }
                NodeType::Constraint => {
                    serde_yaml_ng::to_string(&serde_yaml_ng::from_str::<Constraint>(&original)?)?
                }
                NodeType::Amendment => {
                    serde_yaml_ng::to_string(&serde_yaml_ng::from_str::<Amendment>(&original)?)?
                }
                NodeType::Profile => {
                    serde_yaml_ng::to_string(&serde_yaml_ng::from_str::<Profile>(&original)?)?
                }
            };

            if original != canonical {
                dirty += 1;
                let rel = path
                    .strip_prefix(std::env::current_dir().unwrap_or_default())
                    .unwrap_or(&path);
                if check {
                    let diff = TextDiff::from_lines(&original, &canonical);
                    eprintln!("Diff in {}:", rel.display());
                    for change in diff.iter_all_changes() {
                        match change.tag() {
                            ChangeTag::Delete => eprint!("-{change}"),
                            ChangeTag::Insert => eprint!("+{change}"),
                            ChangeTag::Equal => {}
                        }
                    }
                    eprintln!();
                } else {
                    fs::write(&path, &canonical)?;
                }
            }
        }
    }

    if check && dirty > 0 {
        anyhow::bail!("{dirty} file(s) not in canonical format (run `shapes fmt` to fix)");
    }

    Ok(())
}
