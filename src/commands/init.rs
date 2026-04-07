//! `shapes init` — bootstraps a new `.shapes/` directory in the current
//! working directory.

use std::env;

use anyhow::Result;

use crate::store::FileStore;
use crate::templates::TemplateKind;

/// Initializes a new `.shapes/` store using the chosen template.
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
