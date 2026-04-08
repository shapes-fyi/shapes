//! `shapes create profile` — scaffolds a new Profile node from a
//! starter kit.
//!
//! Unlike `shapes create shape/constraint`, this command does **not**
//! read from the active profile. A new profile needs seed data that
//! only a [`StarterKit`](crate::templates::StarterKit) can supply,
//! because profiles are the governance source and there is nothing
//! "more fundamental" to read from.

use anyhow::Result;

use crate::OutputFormat;
use crate::commands::shared::{read_from, report_created};
use crate::model::{NodeType, Profile, ProfileId};
use crate::store::{FileStore, NodeStore};
use crate::templates::KitKind;

/// Field bag for `shapes create profile`.
pub struct CreateProfileArgs {
    /// `--name` value.
    pub name: Option<String>,
    /// Optional per-call `--kit`. Defaults to [`KitKind::Software`]
    /// when the user does not specify one.
    pub kit: Option<KitKind>,
    /// Optional `--from` path or `-` for stdin.
    pub from: Option<String>,
}

/// Creates a new Profile node and writes it to the store.
pub fn create_profile(
    store: &FileStore,
    args: CreateProfileArgs,
    id_only: bool,
    format: OutputFormat,
) -> Result<()> {
    let id = ProfileId::new(store.next_id(NodeType::Profile)?);

    if let Some(path) = args.from {
        let content = read_from(&path)?;
        let mut p: Profile = serde_yml::from_str(&content)?;
        p.id = id;
        let saved_path = store.save(NodeType::Profile, id.get(), &p)?;
        report_created(id_only, &id.to_string(), &saved_path, &p, format)?;
        return Ok(());
    }

    // clap requires --name when --from is absent.
    let name = args
        .name
        .expect("clap requires --name when --from is absent");
    let kind = args.kit.unwrap_or(KitKind::Software);
    let kit = kind.kit();
    let profile = kit.build_profile(id, &name);
    let saved_path = store.save(NodeType::Profile, id.get(), &profile)?;
    if id_only {
        println!("{id}");
    } else {
        eprintln!("Created {}", saved_path.display());
        // Re-serialize for display so the caller sees exactly what
        // landed on disk.
        print!("{}", serde_yml::to_string(&profile)?);
    }
    Ok(())
}
