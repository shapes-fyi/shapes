use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::model::NodeType;

const SHAPES_DIR: &str = ".shapes";
const META_FILE: &str = "meta.yaml";

// ---------------------------------------------------------------------------
// Meta — tracks next IDs and spec version
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub version: String,
    pub next_ids: NextIds,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NextIds {
    pub shape: u64,
    pub constraint: u64,
    pub amendment: u64,
    pub profile: u64,
}

impl Meta {
    fn new() -> Self {
        Meta {
            version: "0.1.0".into(),
            next_ids: NextIds {
                shape: 1,
                constraint: 1,
                amendment: 1,
                profile: 1,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Store — file-based .shapes/ directory operations
// ---------------------------------------------------------------------------

pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Open an existing .shapes/ store in the given directory.
    pub fn open(dir: &Path) -> Result<Self> {
        let root = dir.join(SHAPES_DIR);
        if !root.is_dir() {
            bail!("No .shapes/ directory found. Run `shapes init` first.");
        }
        Ok(Store { root })
    }

    /// Initialize a new .shapes/ store in the given directory.
    pub fn init(dir: &Path) -> Result<Self> {
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

        let meta = Meta::new();
        let meta_path = root.join(META_FILE);
        let yaml = serde_yaml::to_string(&meta)?;
        fs::write(&meta_path, yaml).context("failed to write meta.yaml")?;

        Ok(Store { root })
    }

    // -- Meta operations ----------------------------------------------------

    fn meta_path(&self) -> PathBuf {
        self.root.join(META_FILE)
    }

    fn load_meta(&self) -> Result<Meta> {
        let content = fs::read_to_string(self.meta_path()).context("failed to read meta.yaml")?;
        Ok(serde_yaml::from_str(&content)?)
    }

    fn save_meta(&self, meta: &Meta) -> Result<()> {
        let yaml = serde_yaml::to_string(meta)?;
        fs::write(self.meta_path(), yaml).context("failed to write meta.yaml")
    }

    /// Allocate and return the next ID for the given node type.
    pub fn next_id(&self, node_type: NodeType) -> Result<u64> {
        let mut meta = self.load_meta()?;
        let id = match node_type {
            NodeType::Shape => &mut meta.next_ids.shape,
            NodeType::Constraint => &mut meta.next_ids.constraint,
            NodeType::Amendment => &mut meta.next_ids.amendment,
            NodeType::Profile => &mut meta.next_ids.profile,
        };
        let current = *id;
        *id += 1;
        self.save_meta(&meta)?;
        Ok(current)
    }

    // -- Node CRUD ----------------------------------------------------------

    fn node_path(&self, node_type: NodeType, id: u64) -> PathBuf {
        self.root
            .join(node_type.dir_name())
            .join(format!("{id}.yaml"))
    }

    /// Save a node to disk as YAML.
    pub fn save<T: Serialize>(&self, node_type: NodeType, id: u64, node: &T) -> Result<()> {
        let path = self.node_path(node_type, id);
        let yaml = serde_yaml::to_string(node)?;
        fs::write(&path, yaml)
            .with_context(|| format!("failed to write {}", path.display()))
    }

    /// Load a single node by type and ID.
    pub fn load<T: DeserializeOwned>(&self, node_type: NodeType, id: u64) -> Result<T> {
        let path = self.node_path(node_type, id);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("{} {} not found", node_type, id))?;
        Ok(serde_yaml::from_str(&content)?)
    }

    /// List all IDs for a given node type (sorted).
    pub fn list_ids(&self, node_type: NodeType) -> Result<Vec<u64>> {
        let dir = self.root.join(node_type.dir_name());
        let mut ids = Vec::new();
        for entry in fs::read_dir(&dir)
            .with_context(|| format!("failed to read {}", dir.display()))?
        {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(stem) = name.strip_suffix(".yaml")
                && let Ok(id) = stem.parse::<u64>()
            {
                ids.push(id);
            }
        }
        ids.sort();
        Ok(ids)
    }

    /// Return the file path for a node (for display to user).
    pub fn node_file_path(&self, node_type: NodeType, id: u64) -> PathBuf {
        self.node_path(node_type, id)
    }
}
