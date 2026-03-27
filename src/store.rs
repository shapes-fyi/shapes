use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::model::NodeType;

const SHAPES_DIR: &str = ".shapes";
const META_FILE: &str = "meta.yaml";

// ---------------------------------------------------------------------------
// Meta — protocol version marker
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub version: String,
}

impl Meta {
    fn new() -> Self {
        Meta {
            version: "0.1.0".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Minimal struct to extract just the id from a YAML file
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct IdOnly {
    id: u64,
}

#[derive(Deserialize)]
struct IdAndName {
    #[allow(dead_code)]
    id: u64,
    name: String,
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

    // -- ID allocation ------------------------------------------------------

    /// Allocate the next ID for a node type by scanning existing nodes.
    pub fn next_id(&self, node_type: NodeType) -> Result<u64> {
        let ids = self.list_ids(node_type)?;
        Ok(ids.into_iter().max().unwrap_or(0) + 1)
    }

    // -- Directory scanning -------------------------------------------------

    /// Return the directory for a given node type.
    fn type_dir(&self, node_type: NodeType) -> PathBuf {
        self.root.join(node_type.dir_name())
    }

    /// List all .yaml files in a node type directory.
    fn yaml_files(&self, node_type: NodeType) -> Result<Vec<PathBuf>> {
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

    /// Find the file path for a node with the given id.
    fn find_file(&self, node_type: NodeType, id: u64) -> Result<PathBuf> {
        for path in self.yaml_files(node_type)? {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            if let Ok(parsed) = serde_yaml::from_str::<IdOnly>(&content) {
                if parsed.id == id {
                    return Ok(path);
                }
            }
        }
        bail!("{} {} not found", node_type, id)
    }

    // -- Node CRUD ----------------------------------------------------------

    /// Save a node to disk as YAML, generating a descriptive filename.
    pub fn save<T: Serialize>(&self, node_type: NodeType, id: u64, node: &T) -> Result<PathBuf> {
        // If a file with this id already exists, overwrite it.
        if let Ok(existing) = self.find_file(node_type, id) {
            let yaml = serde_yaml::to_string(node)?;
            fs::write(&existing, yaml)
                .with_context(|| format!("failed to write {}", existing.display()))?;
            return Ok(existing);
        }

        // New node — serialize to extract the name for the filename.
        let yaml = serde_yaml::to_string(node)?;
        let slug = if let Ok(parsed) = serde_yaml::from_str::<IdAndName>(&yaml) {
            slugify(&parsed.name)
        } else {
            id.to_string()
        };

        let path = self.type_dir(node_type).join(format!("{slug}.yaml"));
        fs::write(&path, &yaml).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(path)
    }

    /// Load a single node by type and ID.
    pub fn load<T: DeserializeOwned>(&self, node_type: NodeType, id: u64) -> Result<T> {
        let path = self.find_file(node_type, id)?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Ok(serde_yaml::from_str(&content)?)
    }

    /// List all IDs for a given node type (sorted).
    pub fn list_ids(&self, node_type: NodeType) -> Result<Vec<u64>> {
        let mut ids = Vec::new();
        for path in self.yaml_files(node_type)? {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            if let Ok(parsed) = serde_yaml::from_str::<IdOnly>(&content) {
                ids.push(parsed.id);
            }
        }
        ids.sort();
        Ok(ids)
    }
}

// ---------------------------------------------------------------------------
// Slugify — turn a name into a filename-safe slug
// ---------------------------------------------------------------------------

fn slugify(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse multiple hyphens, trim leading/trailing
    let mut result = String::new();
    let mut prev_hyphen = true; // treat start as hyphen to trim leading
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
    // Trim trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }
    if result.is_empty() {
        "node".into()
    } else {
        result
    }
}
