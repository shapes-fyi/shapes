//! `shapes create` — verb-level dispatcher for the four `create`
//! noun branches (`shape`, `constraint`, `amendment`, `profile`).
//!
//! Each noun lives in its own sibling file; this module is wiring
//! only.

pub mod amendment;
pub mod constraint;
pub mod profile;
pub mod profile_helpers;
pub mod shape;

use anyhow::Result;

use crate::CreateCommand;
use crate::OutputFormat;
use crate::commands::shared::open_store;

use self::amendment::{CreateAmendmentArgs, create_amendment};
use self::constraint::{CreateConstraintArgs, create_constraint};
use self::profile::{CreateProfileArgs, create_profile};
use self::shape::{CreateShapeArgs, create_shape};

/// Dispatches a [`CreateCommand`] to the appropriate per-noun handler.
pub fn create(cmd: CreateCommand, id_only: bool, format: OutputFormat) -> Result<()> {
    let store = open_store()?;
    match cmd {
        CreateCommand::Shape {
            name,
            kind,
            summary,
            source,
            profile,
            description,
            template,
            from,
        } => create_shape(
            &store,
            CreateShapeArgs {
                name,
                kind,
                summary,
                source,
                profile,
                description,
                template,
                from,
            },
            id_only,
            format,
        ),
        CreateCommand::Constraint {
            name,
            kind,
            rule,
            enforcement,
            summary,
            source,
            intent_kind,
            profile,
            description,
            template,
            from,
        } => create_constraint(
            &store,
            CreateConstraintArgs {
                name,
                kind,
                rule,
                enforcement,
                summary,
                source,
                intent_kind,
                profile,
                description,
                template,
                from,
            },
            id_only,
            format,
        ),
        CreateCommand::Amendment {
            name,
            target_shapes,
            target_constraints,
            summary,
            source,
            version_impact,
            description,
            from,
        } => create_amendment(
            &store,
            CreateAmendmentArgs {
                name,
                target_shapes,
                target_constraints,
                summary,
                source,
                version_impact,
                description,
                from,
            },
            id_only,
            format,
        ),
        CreateCommand::Profile {
            name,
            summary,
            source,
            amendment_model,
            description,
            template,
            from,
        } => create_profile(
            &store,
            CreateProfileArgs {
                name,
                summary,
                source,
                amendment_model,
                description,
                template,
                from,
            },
            id_only,
            format,
        ),
    }
}
