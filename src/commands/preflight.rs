//! `shapes preflight` — prints skill preamble (version, update check,
//! shape tree) for Claude Code skill preprocessing blocks where shell
//! expansions are forbidden.

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::DagType;
use crate::commands::tree;

/// Prints the skill preamble to stdout.
///
/// In normal mode (the default), requires `.shapes/meta.yaml` to exist
/// and prints the shape tree.  In `--init` mode, the store is optional —
/// if it exists, a note and tree are shown; if not, the output is just
/// the version line.
pub fn preflight(init: bool) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");

    // Best-effort update check — never fails the command.
    let latest = check_latest_version();

    println!("Shapes CLI v{version}");
    if let Some(ref latest) = latest {
        if latest != version {
            println!("UPDATE AVAILABLE: v{latest} — run: cargo install shapes-cli");
        }
    }

    let store_exists = Path::new(".shapes/meta.yaml").is_file();

    if init {
        if store_exists {
            println!();
            println!("NOTE: .shapes/ already exists in this project.");
            tree(DagType::Shape, None, 10)?;
        }
    } else if !store_exists {
        println!();
        println!("No .shapes/ directory found in this project.");
        println!("Run /shapes:shapes-init to bootstrap the shapes graph.");
    } else {
        println!();
        tree(DagType::Shape, None, 10)?;
    }

    Ok(())
}

/// Shell out to `curl` to fetch the latest published version from
/// crates.io. Returns `None` on any failure — network errors, timeouts,
/// parse issues — so the caller can silently skip the update notice.
fn check_latest_version() -> Option<String> {
    let output = Command::new("curl")
        .args([
            "-sL",
            "--max-time",
            "3",
            "https://crates.io/api/v1/crates/shapes-cli",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body = String::from_utf8(output.stdout).ok()?;
    let marker = "\"max_version\":\"";
    let start = body.find(marker)? + marker.len();
    let end = body[start..].find('"')? + start;
    Some(body[start..end].to_string())
}
