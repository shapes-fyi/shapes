//! CLI command handlers, one verb per file (or per directory for the
//! multi-noun verbs). This module is wiring only — the actual logic
//! lives in the sibling files listed below.
//!
//! Verb layout:
//!
//! - [`mod@init`] — `shapes init`
//! - [`mod@create`] — `shapes create {shape, constraint, amendment, profile}`
//! - [`mod@get`] — `shapes get`
//! - [`mod@list`] — `shapes list`
//! - [`mod@tree`] — `shapes tree`
//! - [`mod@query`] — `shapes query`
//! - [`mod@validate`] — `shapes validate`
//! - [`mod@ci_check`] — `shapes ci-check`
//! - [`mod@amendment`] — `shapes amendment {archive, unarchive}`
//!
//! Plus the verb-internal helper modules [`scaffold`] (YAML scaffold
//! writers), [`shared`] (output helpers, store opening), and [`dag`]
//! (graph traversal primitives).

mod amendment;
mod ci_check;
mod create;
mod dag;
mod get;
mod init;
mod list;
mod query;
mod scaffold;
mod shared;
mod tree;
mod validate;

pub use amendment::{archive as amendment_archive, unarchive as amendment_unarchive};
pub use ci_check::ci_check;
pub use create::create;
pub use get::get;
pub use init::init;
pub use list::list;
pub use query::query;
pub use tree::tree;
pub use validate::validate;
