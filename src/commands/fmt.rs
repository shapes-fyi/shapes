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
                NodeType::Shape => serde_yml::to_string(&serde_yml::from_str::<Shape>(&original)?)?,
                NodeType::Constraint => {
                    serde_yml::to_string(&serde_yml::from_str::<Constraint>(&original)?)?
                }
                NodeType::Amendment => {
                    serde_yml::to_string(&serde_yml::from_str::<Amendment>(&original)?)?
                }
                NodeType::Profile => {
                    serde_yml::to_string(&serde_yml::from_str::<Profile>(&original)?)?
                }
            };

            if original != canonical {
                dirty += 1;
                let rel = path
                    .strip_prefix(std::env::current_dir().unwrap_or_default())
                    .unwrap_or(&path);
                if check {
                    eprintln!("Diff in {}:", rel.display());
                    let orig_lines: Vec<&str> = original.lines().collect();
                    let canon_lines: Vec<&str> = canonical.lines().collect();
                    let max = orig_lines.len().max(canon_lines.len());
                    for i in 0..max {
                        match (orig_lines.get(i), canon_lines.get(i)) {
                            (Some(o), Some(c)) if o != c => {
                                eprintln!("-{o}");
                                eprintln!("+{c}");
                            }
                            (Some(o), None) => eprintln!("-{o}"),
                            (None, Some(c)) => eprintln!("+{c}"),
                            _ => {}
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
