//! On-disk persistence layer for the shapes graph.
//!
//! Defines the [`NodeStore`] read-side abstraction (so commands and
//! tests can swap in alternative backends) and the [`FileStore`]
//! implementation that reads and writes the canonical `.shapes/`
//! directory of YAML files. Also home to the [`Meta`] document and the
//! filename slugifier. See constraint 15 (NodeStore Trait Boundary) in
//! `.shapes/`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::model::{NodeType, ProfileId};
use crate::templates::StarterKit;

const SHAPES_DIR: &str = ".shapes";
const META_FILE: &str = "meta.yaml";

/// Current `.shapes/` store schema version.
///
/// Written into `meta.yaml` by `shapes init` and enforced by
/// [`crate::commands::shared::open_store`]. Stores at an older version
/// must be upgraded via `shapes migrate`.
pub(crate) const CURRENT_STORE_VERSION: &str = "0.2.0";

/// Read-side abstraction over a shapes graph store.
///
/// Concrete implementations (currently [`FileStore`]) decide where the
/// data lives. Commands take `&impl NodeStore` so they remain trivially
/// testable against in-memory fakes.
pub trait NodeStore {
    /// Loads a single node by type and ID.
    fn load<T: DeserializeOwned>(&self, node_type: NodeType, id: u64) -> Result<T>;

    /// Lists all IDs for a given node type, in ascending order.
    fn list_ids(&self, node_type: NodeType) -> Result<Vec<u64>>;

    /// Loads a [`Shape`](crate::model::Shape) by typed ID.
    fn load_shape(&self, id: crate::model::ShapeId) -> Result<crate::model::Shape> {
        self.load(NodeType::Shape, id.get())
    }

    /// Loads a [`Constraint`](crate::model::Constraint) by typed ID.
    fn load_constraint(&self, id: crate::model::ConstraintId) -> Result<crate::model::Constraint> {
        self.load(NodeType::Constraint, id.get())
    }

    /// Loads an [`Amendment`](crate::model::Amendment) by typed ID.
    fn load_amendment(&self, id: crate::model::AmendmentId) -> Result<crate::model::Amendment> {
        self.load(NodeType::Amendment, id.get())
    }

    /// Loads a [`Profile`](crate::model::Profile) by typed ID.
    fn load_profile(&self, id: crate::model::ProfileId) -> Result<crate::model::Profile> {
        self.load(NodeType::Profile, id.get())
    }

    /// Allocates the next free ID for `node_type` by scanning existing
    /// nodes. The default implementation walks `list_ids` and returns
    /// `max + 1`.
    fn next_id(&self, node_type: NodeType) -> Result<u64> {
        let ids = self.list_ids(node_type)?;
        // `unwrap_or(0)` is safe because the only failure mode of
        // `Iterator::max` on a `Vec<u64>` is "the vec was empty",
        // which we handle explicitly here.
        Ok(ids.into_iter().max().unwrap_or(0) + 1)
    }
}

/// On-disk metadata document for a `.shapes/` store.
///
/// Records the spec version the store was written against and the
/// identifier of the **active profile** — the single profile that
/// governs every `shapes create` in this store by default. The
/// active profile is seeded at `shapes init` time from a built-in
/// starter kit and may be edited or replaced freely afterwards.
#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    /// Spec version this store conforms to.
    pub version: String,
    /// ID of the active [`crate::model::Profile`] node.
    pub active_profile: ProfileId,
}

impl Meta {
    fn new(active_profile: ProfileId) -> Self {
        Meta {
            version: CURRENT_STORE_VERSION.into(),
            active_profile,
        }
    }
}

/// Helper struct used to extract just the `id` field from a YAML node.
#[derive(Deserialize)]
pub(crate) struct IdOnly {
    pub(crate) id: u64,
}

/// Helper struct used to extract the `name` field for filename
/// generation. Unknown fields (like `id`) are ignored by serde.
#[derive(Deserialize)]
struct JustName {
    name: String,
}

/// File-backed [`NodeStore`] rooted at a `.shapes/` directory.
pub struct FileStore {
    root: PathBuf,
}

impl FileStore {
    /// Opens an existing `.shapes/` store anchored at `dir`.
    pub fn open(dir: &Path) -> Result<Self> {
        let root = dir.join(SHAPES_DIR);
        if !root.is_dir() {
            bail!("No .shapes/ directory found. Run `shapes init` first.");
        }
        Ok(FileStore { root })
    }

    /// Initializes a new `.shapes/` store under `dir`, seeding the
    /// active profile from the given starter kit.
    ///
    /// Writes:
    ///
    /// 1. `.shapes/` directory + one subdirectory per node type.
    /// 2. A starter profile (id 1) under `.shapes/profiles/`, built
    ///    from `kit.build_profile`.
    /// 3. `.shapes/meta.yaml` pointing `active_profile` at that
    ///    profile.
    pub fn init(dir: &Path, kit: &StarterKit) -> Result<Self> {
        let root = dir.join(SHAPES_DIR);
        if root.is_dir() {
            bail!(".shapes/ directory already exists.");
        }

        fs::create_dir(&root).context("failed to create .shapes/")?;
        for node_type in &[
            NodeType::Shape,
            NodeType::Constraint,
            NodeType::Amendment,
            NodeType::Profile,
        ] {
            fs::create_dir(root.join(node_type.dir_name()))
                .with_context(|| format!("failed to create .shapes/{}/", node_type.dir_name()))?;
        }

        // Seed the starter profile as profile id 1.
        let profile_id = ProfileId::new(1);
        let profile_name = format!("{} starter profile", kit.name);
        let profile_yaml = kit.to_profile_yaml(profile_id, &profile_name);
        let slug = slugify(&profile_name);
        let profile_path = root
            .join(NodeType::Profile.dir_name())
            .join(format!("{}-{slug}.yaml", profile_id.get()));
        fs::write(&profile_path, profile_yaml)
            .with_context(|| format!("failed to write {}", profile_path.display()))?;

        // Point meta.yaml at the seeded profile.
        let meta = Meta::new(profile_id);
        let meta_path = root.join(META_FILE);
        let yaml = serde_yaml_ng::to_string(&meta)?;
        fs::write(&meta_path, yaml).context("failed to write meta.yaml")?;

        Ok(FileStore { root })
    }

    /// Reads the store's `meta.yaml`. Returns the parsed [`Meta`] or
    /// an error if the file is missing or malformed.
    pub fn read_meta(&self) -> Result<Meta> {
        let path = self.root.join(META_FILE);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Ok(serde_yaml_ng::from_str(&content)?)
    }

    /// Writes the store's `meta.yaml`. Used by `shapes migrate` to bump
    /// the schema version after each successful migration step.
    pub fn write_meta(&self, meta: &Meta) -> Result<()> {
        let path = self.root.join(META_FILE);
        let yaml = serde_yaml_ng::to_string(meta)?;
        fs::write(&path, yaml).context("failed to write meta.yaml")
    }

    /// Saves a node by writing a pre-formatted YAML string directly.
    ///
    /// Used by the scaffold writers, which emit YAML with comments and
    /// `TODO:` placeholders that would be lost if round-tripped through
    /// serde. The caller is responsible for ensuring `content` parses
    /// as a node of `node_type` with the given `id` and `name`.
    pub fn save_raw(
        &self,
        node_type: NodeType,
        id: u64,
        name: &str,
        content: &str,
    ) -> Result<PathBuf> {
        if let Ok(existing) = self.find_file(node_type, id) {
            fs::write(&existing, content)
                .with_context(|| format!("failed to write {}", existing.display()))?;
            return Ok(existing);
        }
        let slug = slugify(name);
        let path = self.type_dir(node_type).join(format!("{id}-{slug}.yaml"));
        fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(path)
    }

    /// Saves a node to disk as YAML, generating a descriptive filename
    /// from the node's `name` field.
    pub fn save<T: Serialize>(&self, node_type: NodeType, id: u64, node: &T) -> Result<PathBuf> {
        // If a file with this id already exists, overwrite it.
        if let Ok(existing) = self.find_file(node_type, id) {
            let yaml = serde_yaml_ng::to_string(node)?;
            fs::write(&existing, yaml)
                .with_context(|| format!("failed to write {}", existing.display()))?;
            return Ok(existing);
        }

        // New node — serialize to extract the name for the filename.
        let yaml = serde_yaml_ng::to_string(node)?;
        let slug = if let Ok(parsed) = serde_yaml_ng::from_str::<JustName>(&yaml) {
            slugify(&parsed.name)
        } else {
            id.to_string()
        };

        let path = self.type_dir(node_type).join(format!("{id}-{slug}.yaml"));
        fs::write(&path, &yaml).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(path)
    }

    fn type_dir(&self, node_type: NodeType) -> PathBuf {
        self.root.join(node_type.dir_name())
    }

    pub(crate) fn yaml_files(&self, node_type: NodeType) -> Result<Vec<PathBuf>> {
        let dir = self.type_dir(node_type);
        let mut files = Vec::new();
        for entry in
            fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml") {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    pub(crate) fn find_file(&self, node_type: NodeType, id: u64) -> Result<PathBuf> {
        for path in self.yaml_files(node_type)? {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            if let Ok(parsed) = serde_yaml_ng::from_str::<IdOnly>(&content)
                && parsed.id == id
            {
                return Ok(path);
            }
        }
        bail!("{} {} not found", node_type, id)
    }
}

impl NodeStore for FileStore {
    fn load<T: DeserializeOwned>(&self, node_type: NodeType, id: u64) -> Result<T> {
        let path = self.find_file(node_type, id)?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Ok(serde_yaml_ng::from_str(&content)?)
    }

    fn list_ids(&self, node_type: NodeType) -> Result<Vec<u64>> {
        let mut ids = Vec::new();
        for path in self.yaml_files(node_type)? {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            if let Ok(parsed) = serde_yaml_ng::from_str::<IdOnly>(&content) {
                ids.push(parsed.id);
            }
        }
        ids.sort();
        Ok(ids)
    }
}

/// Turns a node `name` into a filename-safe slug: lowercase alphanumerics
/// with all other runs collapsed into single hyphens, and leading or
/// trailing hyphens trimmed. Empty results fall back to `node`.
fn slugify(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse multiple hyphens; treat start as hyphen so leading
    // hyphens are trimmed.
    let mut result = String::new();
    let mut prev_hyphen = true;
    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push('-');
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }
    if result.ends_with('-') {
        result.pop();
    }
    if result.is_empty() {
        "node".into()
    } else {
        result
    }
}
