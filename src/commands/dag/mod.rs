//! DAG traversal and validation primitives shared by the
//! [`tree`], [`query`](crate::commands::query), and
//! [`validate`](crate::commands::validate) commands.

mod tree;
mod validate;
mod walk;

pub use tree::print_tree;
pub use validate::validate;
pub use walk::{ancestors, descendants, effective_constraints, shapes_for_constraint};

#[cfg(test)]
mod tests;
