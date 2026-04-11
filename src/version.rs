//! Type-safe schema version for `.shapes/` stores.
//!
//! [`StoreVersion`] is a parsed `major.minor.patch` triple used in
//! `meta.yaml` and by the migration engine. Modeling versions as a
//! struct — rather than juggling `String` values — gives us total
//! ordering for free (so we can ask "is the store older than
//! current?"), catches malformed version strings at parse time
//! instead of at comparison time, and prevents typos like
//! `"0.02.0" != "0.2.0"` from silently skipping migrations.

use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Errors returned when parsing a [`StoreVersion`] from a string.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoreVersionParseError {
    /// The input did not contain exactly three dot-separated components.
    #[error(
        "invalid store version '{input}': expected three dot-separated components (major.minor.patch)"
    )]
    InvalidShape {
        /// The raw input that failed to parse.
        input: String,
    },
    /// One of the components was not a valid non-negative integer.
    #[error("invalid store version '{input}': {component} component is not a number ({source})")]
    InvalidComponent {
        /// The raw input that failed to parse.
        input: String,
        /// Which component failed: `major`, `minor`, or `patch`.
        component: &'static str,
        /// The underlying integer parse error.
        #[source]
        source: ParseIntError,
    },
}

/// A parsed `major.minor.patch` store schema version.
///
/// Total ordering follows numeric precedence of the components (as in
/// SemVer's main release triple) so `0.1.9 < 0.2.0 < 0.10.0`. This
/// keeps the migration runner correct even when version numbers cross
/// a two-digit boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreVersion {
    /// Major version component.
    pub major: u32,
    /// Minor version component.
    pub minor: u32,
    /// Patch version component.
    pub patch: u32,
}

impl StoreVersion {
    /// Constructs a [`StoreVersion`] at compile time.
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for StoreVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for StoreVersion {
    type Err = StoreVersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');
        let parse_component = |raw: Option<&str>,
                               component: &'static str|
         -> Result<u32, StoreVersionParseError> {
            let raw = raw.ok_or_else(|| StoreVersionParseError::InvalidShape {
                input: s.to_string(),
            })?;
            raw.parse::<u32>()
                .map_err(|source| StoreVersionParseError::InvalidComponent {
                    input: s.to_string(),
                    component,
                    source,
                })
        };

        let major = parse_component(parts.next(), "major")?;
        let minor = parse_component(parts.next(), "minor")?;
        let patch = parse_component(parts.next(), "patch")?;

        if parts.next().is_some() {
            return Err(StoreVersionParseError::InvalidShape {
                input: s.to_string(),
            });
        }

        Ok(Self::new(major, minor, patch))
    }
}

impl Serialize for StoreVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for StoreVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_well_formed_version() {
        let v: StoreVersion = "0.2.0".parse().unwrap();
        assert_eq!(v, StoreVersion::new(0, 2, 0));
    }

    #[test]
    fn display_roundtrips() {
        let v = StoreVersion::new(1, 20, 300);
        assert_eq!(v.to_string(), "1.20.300");
        assert_eq!(v.to_string().parse::<StoreVersion>().unwrap(), v);
    }

    #[test]
    fn rejects_too_few_components() {
        assert!(matches!(
            "0.2".parse::<StoreVersion>(),
            Err(StoreVersionParseError::InvalidShape { .. })
        ));
    }

    #[test]
    fn rejects_too_many_components() {
        assert!(matches!(
            "0.2.0.1".parse::<StoreVersion>(),
            Err(StoreVersionParseError::InvalidShape { .. })
        ));
    }

    #[test]
    fn rejects_non_numeric_component() {
        assert!(matches!(
            "0.x.0".parse::<StoreVersion>(),
            Err(StoreVersionParseError::InvalidComponent {
                component: "minor",
                ..
            })
        ));
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(
            "".parse::<StoreVersion>(),
            Err(StoreVersionParseError::InvalidComponent {
                component: "major",
                ..
            })
        ));
    }

    #[test]
    fn numeric_ordering_handles_two_digit_components() {
        let v019 = StoreVersion::new(0, 1, 9);
        let v020 = StoreVersion::new(0, 2, 0);
        let v0100 = StoreVersion::new(0, 10, 0);

        assert!(v019 < v020);
        assert!(v020 < v0100);
        // String comparison would give "0.10.0" < "0.2.0" — verify the
        // struct ordering disagrees and is numerically correct.
        assert!(v020.to_string() > v0100.to_string());
    }

    #[test]
    fn serde_roundtrip_through_yaml() {
        let v = StoreVersion::new(0, 2, 0);
        let yaml = serde_yaml_ng::to_string(&v).unwrap();
        let parsed: StoreVersion = serde_yaml_ng::from_str(&yaml).unwrap();
        assert_eq!(parsed, v);
    }
}
