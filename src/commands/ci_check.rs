//! `shapes ci-check` — PR-level enforcement on top of `shapes validate`.
//!
//! Designed to run inside a CI job (or locally before committing) and
//! enforce three rules that depend on the **diff** between a base ref
//! and the working tree:
//!
//! - **CI-001**: when invoked with `--require-shapes-changes`, the PR
//!   must touch at least one file under the shapes directory.
//! - **CI-002**: when a shape, constraint, or profile that was already
//!   in `promoted` or `canonical` state on the base ref is semantically
//!   modified on HEAD, the PR must contain a new amendment YAML
//!   targeting that node. Deletions of promoted/canonical nodes also
//!   require an amendment.
//! - **CI-003**: existing amendment files in the diff must not be
//!   modified — amendments are immutable per constraint:10. The first
//!   machine enforcement of that long-standing canonical rule.
//!
//! Comparison semantics: `base` ref vs the **working tree**, so the
//! same command yields the same answer locally (before `git commit`)
//! and in CI (after `actions/checkout@v4`, where the working tree
//! equals HEAD).
//!
//! ## Monitored field set (the heart of CI-002)
//!
//! Only "semantic" field changes trigger CI-002. The realization-
//! refresh exemption (file paths, summaries, evidence, provenance,
//! metadata typo fixes) is intentional: those are mechanical hygiene,
//! not semantic change, and forcing amendments for them would create
//! noise. The full table:
//!
//! | Node | Monitored | Not monitored |
//! |---|---|---|
//! | Shape | name, description, intent, profile, predecessors, constraints, parents, children | id, status, version, realization, evidence, provenance, amendment_log, metadata |
//! | Constraint | name, description, kind, rule, enforcement, intent, profile, parents, children | id, status, version, realization, evidence, provenance, amendment_log, metadata |
//! | Profile | name, description, intent, fields, lifecycle, versioning, amendment_rules | id, status, version, provenance, amendment_log, metadata |

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;

use crate::OutputFormat;
use crate::commands::dag::{Severity, ValidationIssue};
use crate::error::{CiCheckError, CliError};
use crate::model::{Amendment, Constraint, NodeType, Profile, Shape};

/// Runs the PR-level checks against the working tree, comparing each
/// monitored file against its state on `base`.
///
/// `shapes_dir` is the path to the `.shapes/` directory, relative to
/// the current working directory. Defaults are wired up by `main`.
pub fn ci_check(
    base: &str,
    shapes_dir: &Path,
    require_shapes_changes: bool,
    format: OutputFormat,
) -> Result<(), CliError> {
    // Fail fast with a friendly message if `git` isn't on PATH —
    // otherwise the first subprocess explodes with a kernel-level
    // ENOENT and the user has to figure out the cause.
    which::which("git").map_err(|_| {
        CliError::Other(anyhow::anyhow!(
            "shapes ci-check requires `git` on PATH (compares the working tree against {base})",
        ))
    })?;

    let mut issues: Vec<ValidationIssue> = Vec::new();

    // CI-001 — PR must touch the shapes directory at all (opt-in).
    let changed = changed_paths_under(base, shapes_dir).map_err(CliError::Other)?;
    if require_shapes_changes && changed.is_empty() {
        issues.push(ValidationIssue {
            invariant: "CI-001".into(),
            severity: Severity::Error,
            node_type: "pull-request".into(),
            node_id: "-".into(),
            message: format!(
                "PR has no changes under {} (required by --require-shapes-changes)",
                shapes_dir.display()
            ),
        });
    }

    // CI-002 + CI-003 — amendment-required-on-promoted/canonical-change
    // and amendment-immutability. Only meaningful when a shapes
    // directory is actually present at HEAD; if not, skip with a quiet
    // pass instead of erroring out.
    if shapes_dir.is_dir() {
        let satisfied =
            collect_satisfied_targets(base, shapes_dir, &mut issues).map_err(CliError::Other)?;
        check_required_amendments::<Shape>(
            base,
            shapes_dir,
            NodeType::Shape,
            &satisfied.shape_ids,
            shape_monitored_changed,
            &mut issues,
        )
        .map_err(CliError::Other)?;
        check_required_amendments::<Constraint>(
            base,
            shapes_dir,
            NodeType::Constraint,
            &satisfied.constraint_ids,
            constraint_monitored_changed,
            &mut issues,
        )
        .map_err(CliError::Other)?;
        check_required_amendments::<Profile>(
            base,
            shapes_dir,
            NodeType::Profile,
            &satisfied.profile_ids,
            profile_monitored_changed,
            &mut issues,
        )
        .map_err(CliError::Other)?;
    }

    report(&issues, format)
}

/// Set of node ids that have a satisfying amendment in the PR (newly
/// added amendments only — modified amendments fire CI-003 and do not
/// satisfy CI-002).
#[derive(Debug, Default)]
struct SatisfiedTargets {
    shape_ids: BTreeSet<u64>,
    constraint_ids: BTreeSet<u64>,
    profile_ids: BTreeSet<u64>,
}

/// Walks every amendment file under `shapes_dir/amendments/` on disk,
/// classifies each as "new in PR" (file did not exist on `base`) or
/// "modified in PR" (file existed on base, content differs), and
/// returns the union of `targets.{shape_ids, constraint_ids,
/// profile_ids}` from the new amendments. Modified amendments push a
/// CI-003 issue and do NOT contribute to the satisfied set.
fn collect_satisfied_targets(
    base: &str,
    shapes_dir: &Path,
    issues: &mut Vec<ValidationIssue>,
) -> Result<SatisfiedTargets> {
    let mut satisfied = SatisfiedTargets::default();

    let amendment_dir = shapes_dir.join(NodeType::Amendment.dir_name());
    if !amendment_dir.is_dir() {
        return Ok(satisfied);
    }

    let head_files = disk_yaml_files(&amendment_dir)?;
    for path in head_files {
        let head_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let head_amend: Amendment = serde_yml::from_str(&head_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        // Resolve the path relative to repo root for git lookups.
        let rel = repo_relative_path(&path)?;
        let base_text = git_show(base, &rel)?;

        match base_text {
            None => {
                // Newly added amendment — its targets satisfy CI-002.
                satisfied
                    .shape_ids
                    .extend(head_amend.targets.shape_ids.iter().map(|id| id.get()));
                satisfied
                    .constraint_ids
                    .extend(head_amend.targets.constraint_ids.iter().map(|id| id.get()));
                satisfied
                    .profile_ids
                    .extend(head_amend.targets.profile_ids.iter().map(|id| id.get()));
            }
            Some(base_text) => {
                let base_amend: Amendment = serde_yml::from_str(&base_text).with_context(|| {
                    format!("failed to parse base version of {}", rel.display())
                })?;
                if base_amend != head_amend {
                    issues.push(ValidationIssue {
                        invariant: "CI-003".into(),
                        severity: Severity::Error,
                        node_type: "amendment".into(),
                        node_id: head_amend.id.to_string(),
                        message: format!(
                            "amendment {} was modified in this PR — amendments are immutable per constraint:10",
                            rel.display()
                        ),
                    });
                    // Modified amendments do NOT satisfy CI-002.
                }
            }
        }
    }

    Ok(satisfied)
}

/// For one node type, pairs the disk view (HEAD) with the base view
/// by node id and emits CI-002 issues for each promoted/canonical node
/// that was semantically modified or deleted without a satisfying
/// amendment.
fn check_required_amendments<T>(
    base: &str,
    shapes_dir: &Path,
    node_type: NodeType,
    satisfied: &BTreeSet<u64>,
    monitored_changed: fn(&T, &T) -> bool,
    issues: &mut Vec<ValidationIssue>,
) -> Result<()>
where
    T: DeserializeOwned + StatusedNode,
{
    let type_dir = shapes_dir.join(node_type.dir_name());
    let head_map: BTreeMap<u64, T> = if type_dir.is_dir() {
        load_disk_map::<T>(&type_dir)?
    } else {
        BTreeMap::new()
    };
    let base_map: BTreeMap<u64, T> = load_base_map::<T>(base, shapes_dir, node_type)?;

    // Pair by id: handles renames, slug churn, and moves between
    // sibling files automatically.
    let all_ids: BTreeSet<u64> = head_map.keys().chain(base_map.keys()).copied().collect();
    for id in all_ids {
        let base_node = base_map.get(&id);
        let head_node = head_map.get(&id);
        let Some(base_node) = base_node else {
            // New in PR — no requirement.
            continue;
        };
        if !base_node.status_ref().requires_amendment_on_change() {
            continue;
        }
        let needs_amendment = match head_node {
            None => true, // deleted
            Some(head) => monitored_changed(base_node, head),
        };
        if needs_amendment && !satisfied.contains(&id) {
            let kind = match head_node {
                None => "deleted",
                Some(_) => "modified",
            };
            issues.push(ValidationIssue {
                invariant: "CI-002".into(),
                severity: Severity::Error,
                node_type: node_type.to_string(),
                node_id: id.to_string(),
                message: format!(
                    "{node_type} {id} was {kind} on a {} base, but no amendment in this PR targets it",
                    base_node.status_ref().name()
                ),
            });
        }
    }
    Ok(())
}

/// Trait that lets the generic check function reach for `.status`
/// without committing to a specific node type. Implemented for the
/// three node types ci-check enforces.
trait StatusedNode {
    fn status_ref(&self) -> &crate::model::status::Status;
}

impl StatusedNode for Shape {
    fn status_ref(&self) -> &crate::model::status::Status {
        &self.status
    }
}

impl StatusedNode for Constraint {
    fn status_ref(&self) -> &crate::model::status::Status {
        &self.status
    }
}

impl StatusedNode for Profile {
    fn status_ref(&self) -> &crate::model::status::Status {
        &self.status
    }
}

fn shape_monitored_changed(base: &Shape, head: &Shape) -> bool {
    base.name != head.name
        || base.description != head.description
        || base.intent != head.intent
        || base.profile != head.profile
        || base.predecessors != head.predecessors
        || base.constraints != head.constraints
        || base.parents != head.parents
        || base.children != head.children
}

fn constraint_monitored_changed(base: &Constraint, head: &Constraint) -> bool {
    base.name != head.name
        || base.description != head.description
        || base.kind != head.kind
        || base.rule != head.rule
        || base.enforcement != head.enforcement
        || base.intent != head.intent
        || base.profile != head.profile
        || base.parents != head.parents
        || base.children != head.children
}

fn profile_monitored_changed(base: &Profile, head: &Profile) -> bool {
    base.name != head.name
        || base.description != head.description
        || base.intent != head.intent
        || base.fields != head.fields
        || base.lifecycle != head.lifecycle
        || base.versioning != head.versioning
        || base.amendment_rules != head.amendment_rules
}

/// Walks `dir` for top-level `*.yaml` files (non-recursive, matching
/// `FileStore::yaml_files` at src/store.rs:206), parses each, and
/// returns the resulting id → node map.
fn load_disk_map<T: DeserializeOwned>(dir: &Path) -> Result<BTreeMap<u64, T>> {
    let mut map = BTreeMap::new();
    for path in disk_yaml_files(dir)? {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let id = parse_id(&text)
            .with_context(|| format!("failed to read id from {}", path.display()))?;
        let node: T = serde_yml::from_str(&text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        map.insert(id, node);
    }
    Ok(map)
}

/// Enumerates the entries of `<shapes_dir>/<node_type.dir_name()>/`
/// at `base` via `git ls-tree`, then `git show`s each file and
/// parses it. Non-recursive — `git ls-tree` (without `-r`) returns
/// only the immediate children, matching the disk-side walk.
fn load_base_map<T: DeserializeOwned>(
    base: &str,
    shapes_dir: &Path,
    node_type: NodeType,
) -> Result<BTreeMap<u64, T>> {
    let type_dir = shapes_dir.join(node_type.dir_name());
    // git ls-tree wants a path with trailing slash to list directory
    // contents (otherwise it lists the directory entry itself).
    let mut tree_arg = type_dir.as_os_str().to_owned();
    tree_arg.push("/");
    let out = Command::new("git")
        .arg("ls-tree")
        .arg(base)
        .arg("--")
        .arg(&tree_arg)
        .output()
        .with_context(|| format!("failed to spawn `git ls-tree` for {base}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        // Common case: directory does not exist on the base ref —
        // treat as empty rather than erroring.
        if stderr.contains("Not a valid object name") || stderr.is_empty() {
            return Ok(BTreeMap::new());
        }
        bail!("git ls-tree failed: {stderr}");
    }
    let stdout = String::from_utf8_lossy(&out.stdout);

    let mut map = BTreeMap::new();
    for line in stdout.lines() {
        // Format: "<mode> <type> <sha>\t<path>"
        let Some((meta, path_str)) = line.split_once('\t') else {
            continue;
        };
        let parts: Vec<&str> = meta.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let entry_type = parts[1];
        let path = PathBuf::from(path_str);
        if entry_type == "tree" {
            bail!(
                "unexpected nested directory under {}: {} — ci-check matches the FileStore's flat layout",
                type_dir.display(),
                path.display()
            );
        }
        if entry_type != "blob" {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let Some(text) = git_show(base, &path)? else {
            continue;
        };
        let id = parse_id(&text)
            .with_context(|| format!("failed to read id from {}", path.display()))?;
        let node: T = serde_yml::from_str(&text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        map.insert(id, node);
    }
    Ok(map)
}

/// `git show <ref>:<path>` → file contents, or `Ok(None)` when the
/// file does not exist on that ref (git exits 128 with stderr
/// containing the canonical "exists on disk, but not in" /
/// "does not exist" message).
fn git_show(base: &str, path: &Path) -> Result<Option<String>> {
    let mut spec = base.to_string();
    spec.push(':');
    spec.push_str(&path.display().to_string());
    let out = Command::new("git")
        .arg("show")
        .arg(&spec)
        .output()
        .with_context(|| format!("failed to spawn `git show` for {spec}"))?;
    if out.status.success() {
        return Ok(Some(String::from_utf8_lossy(&out.stdout).into_owned()));
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("exists on disk, but not in")
        || stderr.contains("does not exist")
        || stderr.contains("path not in")
    {
        return Ok(None);
    }
    bail!("git show {spec} failed: {stderr}");
}

#[derive(serde::Deserialize)]
struct IdOnly {
    id: u64,
}

fn parse_id(yaml: &str) -> Result<u64> {
    let id_only: IdOnly = serde_yml::from_str(yaml).context("YAML missing top-level `id` field")?;
    Ok(id_only.id)
}

fn disk_yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Resolves `path` (which may be absolute or relative to cwd) to a
/// path relative to the git repo root, suitable for `git show
/// <ref>:<path>`.
fn repo_relative_path(path: &Path) -> Result<PathBuf> {
    let absolute = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", path.display()))?;
    let toplevel = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to spawn `git rev-parse --show-toplevel`")?;
    if !toplevel.status.success() {
        let stderr = String::from_utf8_lossy(&toplevel.stderr);
        bail!("git rev-parse failed: {stderr}");
    }
    let root = PathBuf::from(String::from_utf8_lossy(&toplevel.stdout).trim());
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", root.display()))?;
    let rel = absolute.strip_prefix(&root).with_context(|| {
        format!(
            "{} is not under the git repo root {}",
            absolute.display(),
            root.display()
        )
    })?;
    Ok(rel.to_path_buf())
}

/// Returns the set of paths under `shapes_dir` that differ between
/// `base` and the current working tree. The result includes:
///
/// - tracked files modified, added, deleted, or renamed since `base`
///   (`git diff --name-only <base> -- <shapes_dir>`)
/// - untracked files inside `shapes_dir` (`git ls-files --others
///   --exclude-standard -- <shapes_dir>`)
///
/// Combining both is necessary so the command works locally before a
/// commit (where new files are still untracked) AND in CI after
/// `actions/checkout@v4` (where every PR file is already tracked).
fn changed_paths_under(base: &str, shapes_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();

    // Tracked changes (modified / added / deleted / renamed) vs base.
    let diff = Command::new("git")
        .args(["diff", "--name-only", base, "--"])
        .arg(shapes_dir)
        .output()
        .with_context(|| format!("failed to spawn `git diff` against {base}"))?;
    if !diff.status.success() {
        let stderr = String::from_utf8_lossy(&diff.stderr);
        bail!("git diff failed: {stderr}");
    }
    paths.extend(
        String::from_utf8_lossy(&diff.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(PathBuf::from),
    );

    // Untracked files inside shapes_dir, respecting .gitignore.
    let untracked = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard", "--"])
        .arg(shapes_dir)
        .output()
        .with_context(|| "failed to spawn `git ls-files --others`")?;
    if !untracked.status.success() {
        let stderr = String::from_utf8_lossy(&untracked.stderr);
        bail!("git ls-files failed: {stderr}");
    }
    paths.extend(
        String::from_utf8_lossy(&untracked.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(PathBuf::from),
    );

    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Emits the issue list in the requested format and converts a
/// non-empty list into the dedicated [`CiCheckError::IssuesFound`]
/// error so the CLI returns exit code 2.
fn report(issues: &[ValidationIssue], format: OutputFormat) -> Result<(), CliError> {
    if issues.is_empty() {
        match format {
            OutputFormat::Json => println!("[]"),
            OutputFormat::Yaml => eprintln!("No issues found."),
        }
        return Ok(());
    }
    match format {
        OutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(issues).map_err(|e| CliError::Other(e.into()))?;
            println!("{json}");
        }
        OutputFormat::Yaml => {
            for issue in issues {
                eprintln!("{issue}");
            }
            eprintln!("{} ci-check issue(s) found", issues.len());
        }
    }
    Err(CiCheckError::IssuesFound {
        count: issues.len(),
    }
    .into())
}
