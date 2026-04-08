//! `shapes create profile` — scaffolds a new Profile node.

use anyhow::Result;

use crate::OutputFormat;
use crate::commands::scaffold;
use crate::commands::shared::{read_from, report_created};
use crate::model::{AmendmentModel, NodeType, Profile, ProfileId};
use crate::store::{FileStore, NodeStore};
use crate::templates::TemplateKind;

use super::profile_helpers::resolve_template;

/// Field bag for `shapes create profile`.
pub struct CreateProfileArgs {
    /// `--name` value.
    pub name: Option<String>,
    /// Optional `--summary`.
    pub summary: Option<String>,
    /// `--source` value.
    pub source: String,
    /// `--amendment-model` value.
    pub amendment_model: AmendmentModel,
    /// Optional `--description`.
    pub description: Option<String>,
    /// Optional per-call `--template`.
    pub template: Option<TemplateKind>,
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
    let template = resolve_template(store, args.template)?;
    let amendment_model_str = match args.amendment_model {
        AmendmentModel::Merge => "merge",
        AmendmentModel::Overlay => "overlay",
        AmendmentModel::Edition => "edition",
        AmendmentModel::AppendOnly => "append-only",
    };
    let yaml = scaffold::scaffold_profile(&scaffold::ProfileScaffold {
        id: id.get(),
        name: &name,
        summary: args.summary.as_deref(),
        source: &args.source,
        description: args.description.as_deref(),
        amendment_model: amendment_model_str,
        template,
    });
    let saved_path = store.save_raw(NodeType::Profile, id.get(), &name, &yaml)?;
    if id_only {
        println!("{id}");
    } else {
        eprintln!("Created {}", saved_path.display());
        print!("{yaml}");
    }
    Ok(())
}
