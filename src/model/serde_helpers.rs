//! Reusable serde helpers shared across model types.
//!
//! These deserializers smooth over the YAML quirk that a field may be
//! present-as-null, missing, or present-with-value, and we want all three
//! to land on `T::default()` for collection-like fields.

use serde::{Deserialize, Deserializer};

/// Deserializes a value that may be `null`, missing, or present.
///
/// Maps `null` and missing to `T::default()`. Pair with
/// `#[serde(default, deserialize_with = "null_to_default")]` on `Vec`,
/// `BTreeMap`, and similar fields where the on-disk YAML may legitimately
/// write `field: null` instead of omitting the key.
pub fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
