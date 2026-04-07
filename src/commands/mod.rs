use std::collections::BTreeMap;
use std::env;
use std::io::Read;

use anyhow::Result;
use serde::Serialize;

use crate::error::{CliError, CreateError, ValidationError};
use crate::model::*;
use crate::model::{ShapeId, ConstraintId, AmendmentId, ProfileId, InitiatedType};
use crate::model::profile::FieldSection;
use crate::store::{FileStore, NodeStore};
use crate::templates::{self, Template, TemplateKind};
use crate::{CreateCommand, DagType, OutputFormat, QueryCommand};

mod dag;
mod scaffold;

// ---------------------------------------------------------------------------
// Output helper
// ---------------------------------------------------------------------------

fn output<T: Serialize>(value: &T, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Yaml => {
            print!("{}", serde_yml::to_string(value)?);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
    }
    Ok(())
}

fn read_from(path: &str) -> Result<String> {
    if path == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        Ok(std::fs::read_to_string(path)?)
    }
}

fn open_store() -> Result<FileStore> {
    FileStore::open(&env::current_dir()?)
}

// ---------------------------------------------------------------------------
// Profile helpers for create commands
// ---------------------------------------------------------------------------

fn load_profile(store: &impl NodeStore, profile_id: u64) -> Result<Profile, CreateError> {
    store
        .load::<Profile>(NodeType::Profile, profile_id)
        .map_err(|_| CreateError::ProfileNotFound { id: profile_id })
}

fn validate_kind_against_profile(
    profile: &Profile,
    node_type_str: &str,
    kind: &str,
) -> Result<(), CreateError> {
    let fields = match &profile.fields {
        Some(f) => f,
        None => return Ok(()),
    };
    let section: &FieldSection = match node_type_str {
        "shape" => match &fields.shape {
            Some(s) => s,
            None => return Ok(()),
        },
        "constraint" => match &fields.constraint {
            Some(s) => s,
            None => return Ok(()),
        },
        _ => return Ok(()),
    };
    if let Some(ref group) = section.intent
        && !group.kinds.is_empty()
        && !group.kinds.iter().any(|k| k.name == kind)
    {
        let allowed: Vec<&str> = group.kinds.iter().map(|k| k.name.as_str()).collect();
        return Err(CreateError::InvalidKind {
            kind: kind.to_string(),
            profile_id: profile.id.get(),
            allowed: allowed.join(", "),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

pub fn init(template: TemplateKind) -> Result<()> {
    let dir = env::current_dir()?;
    FileStore::init(&dir, Some(template.as_str()))?;
    let t = template.template();
    eprintln!(
        "Initialized .shapes/ in {} (template: {} — {})",
        dir.display(),
        t.name,
        t.description,
    );
    Ok(())
}

/// Resolve which template to use for a scaffold call. Per-call `--template`
/// wins; otherwise read the active template from `meta.yaml`; otherwise
/// fall back to `software`.
fn resolve_template(store: &FileStore, override_kind: Option<TemplateKind>) -> &'static Template {
    if let Some(k) = override_kind {
        return k.template();
    }
    let meta_template = store.read_meta().ok().and_then(|m| m.template);
    templates::resolve(meta_template.as_deref())
}

// ---------------------------------------------------------------------------
// create
// ---------------------------------------------------------------------------

pub fn create(cmd: CreateCommand, id_only: bool, format: OutputFormat) -> Result<()> {
    let store = open_store()?;

    match cmd {
        CreateCommand::Shape {
            name,
            kind,
            summary,
            source,
            profile,
            description,
            template: template_override,
            from,
        } => {
            let id = ShapeId::new(store.next_id(NodeType::Shape)?);

            if let Some(path) = from {
                let content = read_from(&path)?;
                let mut s: Shape = serde_yml::from_str(&content)?;
                s.id = id;
                let saved_path = store.save(NodeType::Shape, id.get(), &s)?;
                report_created(id_only, &id.to_string(), &saved_path, &s, format)?;
            } else {
                let name = name.expect("clap requires --name when --from is absent");
                let template = resolve_template(&store, template_override);
                let kind_str = kind.unwrap_or_else(|| template.default_shape_kind.to_string());

                if let Some(pid) = profile {
                    let p = load_profile(&store, pid)?;
                    validate_kind_against_profile(&p, "shape", &kind_str)?;
                }

                let yaml = scaffold::scaffold_shape(&scaffold::ShapeScaffold {
                    id,
                    name: &name,
                    kind: &kind_str,
                    summary: summary.as_deref(),
                    source: &source,
                    description: description.as_deref(),
                    profile,
                    template,
                });
                let saved_path = store.save_raw(NodeType::Shape, id.get(), &name, &yaml)?;
                if id_only {
                    println!("{id}");
                } else {
                    eprintln!("Created {}", saved_path.display());
                    print!("{yaml}");
                }
            }
        }

        CreateCommand::Constraint {
            name,
            kind,
            rule,
            enforcement,
            summary,
            source,
            intent_kind,
            profile,
            description,
            template: template_override,
            from,
        } => {
            let id = ConstraintId::new(store.next_id(NodeType::Constraint)?);

            if let Some(path) = from {
                let content = read_from(&path)?;
                let mut c: Constraint = serde_yml::from_str(&content)?;
                c.id = id;
                let saved_path = store.save(NodeType::Constraint, id.get(), &c)?;
                report_created(id_only, &id.to_string(), &saved_path, &c, format)?;
            } else {
                let name = name.expect("clap requires --name when --from is absent");
                let template = resolve_template(&store, template_override);
                let kind_str =
                    kind.unwrap_or_else(|| template.default_constraint_kind.to_string());

                if let Some(pid) = profile {
                    let p = load_profile(&store, pid)?;
                    validate_kind_against_profile(&p, "constraint", &kind_str)?;
                }

                let yaml = scaffold::scaffold_constraint(&scaffold::ConstraintScaffold {
                    id: id.get(),
                    name: &name,
                    kind: &kind_str,
                    rule: rule.as_deref(),
                    enforcement,
                    summary: summary.as_deref(),
                    source: &source,
                    description: description.as_deref(),
                    intent_kind: intent_kind.as_deref(),
                    profile,
                    template,
                });
                let saved_path =
                    store.save_raw(NodeType::Constraint, id.get(), &name, &yaml)?;
                if id_only {
                    println!("{id}");
                } else {
                    eprintln!("Created {}", saved_path.display());
                    print!("{yaml}");
                }
            }
        }

        CreateCommand::Amendment {
            name,
            target_shapes,
            target_constraints,
            summary,
            source,
            version_impact,
            description,
            from,
        } => {
            let id = AmendmentId::new(store.next_id(NodeType::Amendment)?);
            let amendment: Amendment = if let Some(path) = from {
                let content = read_from(&path)?;
                let mut a: Amendment = serde_yml::from_str(&content)?;
                a.id = id;
                a
            } else {
                let name = name.expect("clap requires --name when --from is absent");
                Amendment {
                    id,
                    name: name.clone(),
                    description: description.unwrap_or_else(|| name.clone()),
                    targets: AmendmentTargets {
                        shape_ids: target_shapes,
                        constraint_ids: target_constraints,
                        profile_ids: vec![],
                    },
                    status: Status::proposed(),
                    version_impact,
                    intent: Intent {
                        kind: "amendment".into(),
                        summary: summary.unwrap_or(name),
                        source: serde_yml::Value::String(source),
                        uris: vec![],
                        extra: Default::default(),
                    },
                    constraints: vec![],
                    realization: vec![],
                    evidence: vec![],
                    provenance: vec![],
                    initiated_by: InitiatedBy {
                        initiated_type: InitiatedType::Human,
                        identity: None,
                        provenance: None,
                    },
                    metadata: BTreeMap::new(),
                }
            };
            let path = store.save(NodeType::Amendment, id.get(), &amendment)?;
            if id_only {
                println!("{id}");
            } else {
                eprintln!("Created {}", path.display());
                output(&amendment, format)?;
            }
        }

        CreateCommand::Profile {
            name,
            summary,
            source,
            amendment_model,
            description,
            template: template_override,
            from,
        } => {
            let id = ProfileId::new(store.next_id(NodeType::Profile)?);

            if let Some(path) = from {
                let content = read_from(&path)?;
                let mut p: Profile = serde_yml::from_str(&content)?;
                p.id = id;
                let saved_path = store.save(NodeType::Profile, id.get(), &p)?;
                report_created(id_only, &id.to_string(), &saved_path, &p, format)?;
            } else {
                let name = name.expect("clap requires --name when --from is absent");
                let template = resolve_template(&store, template_override);
                let amendment_model_str = match amendment_model {
                    AmendmentModel::Merge => "merge",
                    AmendmentModel::Overlay => "overlay",
                    AmendmentModel::Edition => "edition",
                    AmendmentModel::AppendOnly => "append-only",
                };
                let yaml = scaffold::scaffold_profile(&scaffold::ProfileScaffold {
                    id: id.get(),
                    name: &name,
                    summary: summary.as_deref(),
                    source: &source,
                    description: description.as_deref(),
                    amendment_model: amendment_model_str,
                    template,
                });
                let saved_path =
                    store.save_raw(NodeType::Profile, id.get(), &name, &yaml)?;
                if id_only {
                    println!("{id}");
                } else {
                    eprintln!("Created {}", saved_path.display());
                    print!("{yaml}");
                }
            }
        }
    }

    Ok(())
}

fn report_created<T: Serialize>(
    id_only: bool,
    id: &str,
    path: &std::path::Path,
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

// ---------------------------------------------------------------------------
// get
// ---------------------------------------------------------------------------

pub fn get(node_type: NodeType, id: u64, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    match node_type {
        NodeType::Shape => output(&store.load::<Shape>(node_type, id)?, format),
        NodeType::Constraint => output(&store.load::<Constraint>(node_type, id)?, format),
        NodeType::Amendment => output(&store.load::<Amendment>(node_type, id)?, format),
        NodeType::Profile => output(&store.load::<Profile>(node_type, id)?, format),
    }
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ListEntry {
    #[serde(rename = "type")]
    node_type: String,
    id: u64,
    name: String,
    status: String,
    kind: String,
}

pub fn list(
    node_type: Option<NodeType>,
    status_filter: Option<String>,
    kind_filter: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let store = open_store()?;
    let types = match node_type {
        Some(t) => vec![t],
        None => vec![
            NodeType::Shape,
            NodeType::Constraint,
            NodeType::Amendment,
            NodeType::Profile,
        ],
    };

    let mut entries = Vec::new();

    for t in types {
        let ids = store.list_ids(t)?;
        for id in ids {
            let (name, status, kind) = match t {
                NodeType::Shape => {
                    let s: Shape = store.load(t, id)?;
                    (s.name, s.status.name().to_string(), s.intent.kind)
                }
                NodeType::Constraint => {
                    let c: Constraint = store.load(t, id)?;
                    (c.name, c.status.name().to_string(), c.kind.clone())
                }
                NodeType::Amendment => {
                    let a: Amendment = store.load(t, id)?;
                    (a.name, a.status.name().to_string(), a.intent.kind)
                }
                NodeType::Profile => {
                    let p: Profile = store.load(t, id)?;
                    (p.name, p.status.name().to_string(), p.intent.kind)
                }
            };

            if let Some(ref sf) = status_filter
                && &status != sf
            {
                continue;
            }
            if let Some(ref kf) = kind_filter
                && &kind != kf
            {
                continue;
            }

            entries.push(ListEntry {
                node_type: t.to_string(),
                id,
                name,
                status,
                kind,
            });
        }
    }

    output(&entries, format)
}

// ---------------------------------------------------------------------------
// tree
// ---------------------------------------------------------------------------

pub fn tree(dag_type: DagType, root: Option<u64>, max_depth: usize) -> Result<()> {
    let store = open_store()?;
    dag::print_tree(&store, dag_type, root, max_depth)
}

// ---------------------------------------------------------------------------
// query
// ---------------------------------------------------------------------------

pub fn query(op: QueryCommand, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    match op {
        QueryCommand::Ancestors { node_type, id } => {
            let result = dag::ancestors(&store, node_type, id)?;
            output(&result, format)
        }
        QueryCommand::Descendants { node_type, id } => {
            let result = dag::descendants(&store, node_type, id)?;
            output(&result, format)
        }
        QueryCommand::Constraints { shape_id } => {
            let result = dag::effective_constraints(&store, shape_id)?;
            output(&result, format)
        }
        QueryCommand::ShapesForConstraint { constraint_id } => {
            let result = dag::shapes_for_constraint(&store, constraint_id)?;
            output(&result, format)
        }
    }
}

// ---------------------------------------------------------------------------
// validate
// ---------------------------------------------------------------------------

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
                let json = serde_json::to_string_pretty(&issues)
                    .map_err(|e| CliError::Other(e.into()))?;
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
