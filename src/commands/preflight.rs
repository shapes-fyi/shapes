//! `shapes preflight` — prints skill preamble (version, update check,
//! schema-drift warning, shape tree) for Claude Code skill preprocessing
//! blocks where shell expansions are forbidden.

use std::cmp::Ordering;
use std::env;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use semver::Version;

use crate::DagType;
use crate::commands::tree;
use crate::store::{CURRENT_STORE_VERSION, FileStore};

/// Prints the skill preamble to stdout.
///
/// In normal mode (the default), requires `.shapes/meta.yaml` to exist
/// and prints the shape tree.  In `--init` mode, the store is optional —
/// if it exists, a note and tree are shown; if not, the output is just
/// the version line.
///
/// If the on-disk store is at a schema version that differs from
/// [`CURRENT_STORE_VERSION`], a drift warning is printed in place of the
/// tree so agents loading a Shapes skill see the migration prompt as the
/// first actionable line in their context window, rather than
/// discovering it later when a gated command bails.
pub fn preflight(init: bool) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");

    // Best-effort update check — never fails the command.
    let latest = check_latest_version();

    println!("Shapes CLI v{version}");
    if let Some(ref latest) = latest
        && latest != version
    {
        println!("UPDATE AVAILABLE: v{latest} — run: cargo install shapes-cli");
    }

    // Schema-drift warning runs BEFORE any tree walk so agents see the
    // migration prompt at the top of every Shapes skill preamble, not
    // hidden inside a later failed command. Short-circuits the rest of
    // the output so the drift message is the sole call to action. The
    // imperative verb comes first (`Run ...`) so an agent focused on its
    // main task can't scroll past the warning as a passive suggestion.
    if let Some(drift) = check_schema_drift() {
        println!();
        match drift {
            SchemaDrift::Outdated(on_disk) => println!(
                "MIGRATION NEEDED: Run `shapes migrate` — .shapes/ store is at version {on_disk} but this CLI expects {CURRENT_STORE_VERSION}."
            ),
            SchemaDrift::Newer(on_disk) => println!(
                "STORE AHEAD OF CLI: Run `cargo install shapes-cli` — .shapes/ store is at version {on_disk} but this CLI only supports up to {CURRENT_STORE_VERSION}."
            ),
        }
        return Ok(());
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

/// One of the two drift states preflight needs to warn about. The
/// equal-version case is represented by `None` at the call site — only
/// states that warrant a warning have variants here.
enum SchemaDrift {
    /// On-disk `meta.version` is older than [`CURRENT_STORE_VERSION`];
    /// resolved by running `shapes migrate`.
    Outdated(Version),
    /// On-disk `meta.version` is newer than [`CURRENT_STORE_VERSION`];
    /// resolved by upgrading the CLI binary. Only reachable when the
    /// user has downgraded their installed `shapes` or is working on a
    /// store written by a future release.
    Newer(Version),
}

/// Best-effort schema-version comparison between `.shapes/meta.yaml`
/// and [`CURRENT_STORE_VERSION`].
///
/// Returns `None` when there is no store, `meta.yaml` is missing or
/// unreadable, or the store is exactly current — so the caller can
/// treat a `Some` as "warn and short-circuit the rest of the preamble".
/// The function deliberately swallows all errors: preflight is a
/// best-effort diagnostic and must never crash a skill load.
fn check_schema_drift() -> Option<SchemaDrift> {
    let store = FileStore::open(&env::current_dir().ok()?).ok()?;
    let meta = store.read_meta().ok()?;
    match meta.version.cmp(&CURRENT_STORE_VERSION) {
        Ordering::Less => Some(SchemaDrift::Outdated(meta.version)),
        Ordering::Greater => Some(SchemaDrift::Newer(meta.version)),
        Ordering::Equal => None,
    }
}

/// Shell out to `curl` to fetch the latest published version from
/// crates.io. Returns `None` on any failure — network errors, timeouts,
/// parse issues — so the caller can silently skip the update notice.
///
/// Respects the `SHAPES_SKIP_UPDATE_CHECK` environment variable: when
/// set to any value, the function returns `None` immediately without
/// touching the network. Used by the integration test suite to pin
/// preflight output to a byte-stable form (no crates.io round-trip, no
/// version-dependent `UPDATE AVAILABLE` line). The variable is opt-in
/// and nothing in the shipping CLI sets it.
fn check_latest_version() -> Option<String> {
    if env::var_os("SHAPES_SKIP_UPDATE_CHECK").is_some() {
        return None;
    }

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
