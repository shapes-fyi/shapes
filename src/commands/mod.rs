use std::collections::BTreeMap;
use std::env;
use std::io::Read;

use anyhow::Result;
use serde::Serialize;

use crate::error::{CliError, CreateError, ValidationError};
use crate::model::*;
use crate::model::{ShapeId, ConstraintId, AmendmentId, ProfileId, InitiatedType};
use crate::model::profile::{FieldGroup, FieldSection};
use crate::store::{FileStore, NodeStore};
use crate::{CreateCommand, DagType, OutputFormat, QueryCommand};

mod dag;

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

fn apply_profile_fields(
    profile: &Profile,
    node_type_str: &str,
    intent_extra: &mut BTreeMap<String, serde_yml::Value>,
    metadata: &mut BTreeMap<String, serde_yml::Value>,
) {
    let fields = match &profile.fields {
        Some(f) => f,
        None => return,
    };
    let section: &FieldSection = match node_type_str {
        "shape" => match &fields.shape {
            Some(s) => s,
            None => return,
        },
        "constraint" => match &fields.constraint {
            Some(s) => s,
            None => return,
        },
        _ => return,
    };
    if let Some(ref group) = section.intent {
        apply_field_group(group, intent_extra);
    }
    if let Some(ref group) = section.metadata {
        apply_field_group(group, metadata);
    }
}

fn apply_field_group(
    group: &FieldGroup,
    map: &mut BTreeMap<String, serde_yml::Value>,
) {
    for fd in &group.fields {
        if map.contains_key(&fd.name) {
            continue;
        }
        if fd.required {
            let placeholder = format!("TODO: {}", fd.description);
            map.insert(fd.name.clone(), serde_yml::Value::String(placeholder));
        }
    }
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

pub fn init() -> Result<()> {
    let dir = env::current_dir()?;
    FileStore::init(&dir)?;
    eprintln!("Initialized .shapes/ in {}", dir.display());
    Ok(())
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
            from,
        } => {
            let id = ShapeId::new(store.next_id(NodeType::Shape)?);
            let shape: Shape = if let Some(path) = from {
                let content = read_from(&path)?;
                let mut s: Shape = serde_yml::from_str(&content)?;
                s.id = id;
                s
            } else {
                let loaded_profile = match profile {
                    Some(pid) => Some(load_profile(&store, pid)?),
                    None => None,
                };

                if let Some(ref p) = loaded_profile {
                    validate_kind_against_profile(p, "shape", &kind)?;
                }

                let mut intent_extra: BTreeMap<String, serde_yml::Value> = BTreeMap::new();
                let mut metadata: BTreeMap<String, serde_yml::Value> = BTreeMap::new();

                if let Some(ref p) = loaded_profile {
                    apply_profile_fields(p, "shape", &mut intent_extra, &mut metadata);
                }

                Shape {
                    id,
                    name: name.clone(),
                    description: description.unwrap_or_else(|| name.clone()),
                    profile: profile.map(ProfileId::new),
                    version: None,
                    predecessors: vec![],
                    status: Status::proposed(),
                    intent: Intent {
                        kind,
                        summary: summary.unwrap_or(name),
                        source: serde_yml::Value::String(source),
                        uris: vec![],
                        extra: intent_extra,
                    },
                    constraints: vec![],
                    realization: vec![],
                    evidence: vec![],
                    provenance: vec![],
                    amendment_log: vec![],
                    parents: vec![],
                    children: vec![],
                    metadata,
                }
            };
            let path = store.save(NodeType::Shape, id.get(), &shape)?;
            if id_only {
                println!("{id}");
            } else {
                eprintln!("Created {}", path.display());
                output(&shape, format)?;
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
            from,
        } => {
            let id = ConstraintId::new(store.next_id(NodeType::Constraint)?);
            let constraint: Constraint = if let Some(path) = from {
                let content = read_from(&path)?;
                let mut c: Constraint = serde_yml::from_str(&content)?;
                c.id = id;
                c
            } else {
                let loaded_profile = match profile {
                    Some(pid) => Some(load_profile(&store, pid)?),
                    None => None,
                };

                if let Some(ref p) = loaded_profile {
                    validate_kind_against_profile(p, "constraint", &kind)?;
                }

                let mut intent_extra: BTreeMap<String, serde_yml::Value> = BTreeMap::new();
                let mut metadata: BTreeMap<String, serde_yml::Value> = BTreeMap::new();

                if let Some(ref p) = loaded_profile {
                    apply_profile_fields(p, "constraint", &mut intent_extra, &mut metadata);
                }

                let resolved_intent_kind = intent_kind.unwrap_or_else(|| kind.clone());
                Constraint {
                    id,
                    name: name.clone(),
                    description: description.unwrap_or_else(|| name.clone()),
                    kind,
                    rule: rule.unwrap_or_default(),
                    enforcement,
                    profile: profile.map(ProfileId::new),
                    version: None,
                    status: Status::proposed(),
                    intent: Intent {
                        kind: resolved_intent_kind,
                        summary: summary.unwrap_or(name),
                        source: serde_yml::Value::String(source),
                        uris: vec![],
                        extra: intent_extra,
                    },
                    realization: vec![],
                    evidence: vec![],
                    provenance: vec![],
                    amendment_log: vec![],
                    parents: vec![],
                    children: vec![],
                    metadata,
                }
            };
            let path = store.save(NodeType::Constraint, id.get(), &constraint)?;
            if id_only {
                println!("{id}");
            } else {
                eprintln!("Created {}", path.display());
                output(&constraint, format)?;
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
            from,
        } => {
            let id = ProfileId::new(store.next_id(NodeType::Profile)?);
            let profile: Profile = if let Some(path) = from {
                let content = read_from(&path)?;
                let mut p: Profile = serde_yml::from_str(&content)?;
                p.id = id;
                p
            } else {
                Profile {
                    id,
                    name: name.clone(),
                    description: description.unwrap_or_else(|| name.clone()),
                    version: None,
                    status: Status::proposed(),
                    intent: Intent {
                        kind: "governance".into(),
                        summary: summary.unwrap_or(name),
                        source: serde_yml::Value::String(source),
                        uris: vec![],
                        extra: Default::default(),
                    },
                    provenance: vec![],
                    lifecycle: None,
                    fields: None,
                    versioning: None,
                    amendment_rules: Some(AmendmentRules {
                        application: amendment_model,
                    }),
                    amendment_log: vec![],
                    metadata: BTreeMap::new(),
                }
            };
            let path = store.save(NodeType::Profile, id.get(), &profile)?;
            if id_only {
                println!("{id}");
            } else {
                eprintln!("Created {}", path.display());
                output(&profile, format)?;
            }
        }
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
