//! CLI command handlers, one verb per file (or per directory for the
//! multi-noun verbs). This module is wiring only — the actual logic
//! lives in the sibling files listed below.
//!
//! Verb layout:
//!
//! - [`init`] — `shapes init`
//! - [`create`] — `shapes create {shape, constraint, amendment, profile}`
//! - [`get`] — `shapes get`
//! - [`list`] — `shapes list`
//! - [`tree`] — `shapes tree`
//! - [`query`] — `shapes query`
//! - [`validate`] — `shapes validate`
//!
//! Plus the verb-internal helper modules [`scaffold`] (YAML scaffold
//! writers), [`shared`] (output helpers, store opening), and [`dag`]
//! (graph traversal primitives).

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

pub use create::create;
pub use get::get;
pub use init::init;
pub use list::list;
pub use query::query;
pub use tree::tree;
pub use validate::validate;
