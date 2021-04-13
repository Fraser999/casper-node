use alloc::vec::Vec;
use core::{convert::TryFrom, fmt, num::ParseIntError};

use datasize::DataSize;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use displaydoc::Display;
#[cfg(feature = "std")]
use thiserror::Error;

use crate::bytesrepr::{self, Error, FromBytes, ToBytes, U32_SERIALIZED_LENGTH};

/// Length of SemVer when serialized
pub const SEM_VER_SERIALIZED_LENGTH: usize = 3 * U32_SERIALIZED_LENGTH;

/// A struct for semantic versioning.
#[derive(
    Copy,
    Clone,
    DataSize,
    Debug,
    Default,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub struct SemVer {
    /// Major version.
    pub major: u32,
    /// Minor version.
    pub minor: u32,
    /// Patch version.
    pub patch: u32,
}

impl SemVer {
    /// Version 1.0.0.
    pub const V1_0_0: SemVer = SemVer {
        major: 1,
        minor: 0,
        patch: 0,
    };

    /// Constructs a new `SemVer` from the given semver parts.
    pub const fn new(major: u32, minor: u32, patch: u32) -> SemVer {
        SemVer {
            major,
            minor,
            patch,
        }
    }
}

impl ToBytes for SemVer {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), bytesrepr::Error> {
        self.major.to_bytes(sink)?;
        self.minor.to_bytes(sink)?;
        self.patch.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        SEM_VER_SERIALIZED_LENGTH
    }
}

impl FromBytes for SemVer {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (major, rem): (u32, &[u8]) = FromBytes::from_bytes(bytes)?;
        let (minor, rem): (u32, &[u8]) = FromBytes::from_bytes(rem)?;
        let (patch, rem): (u32, &[u8]) = FromBytes::from_bytes(rem)?;
        Ok((SemVer::new(major, minor, patch), rem))
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Parsing error when creating a SemVer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Error))]
#[cfg_attr(not(feature = "std"), derive(Display))]
pub enum ParseSemVerError {
    /// Invalid version format
    #[cfg_attr(feature = "std", error("Invalid version format"))]
    InvalidVersionFormat,
    /// {0}
    #[cfg_attr(feature = "std", error("{}", _0))]
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for ParseSemVerError {
    fn from(error: ParseIntError) -> ParseSemVerError {
        ParseSemVerError::ParseIntError(error)
    }
}

impl TryFrom<&str> for SemVer {
    type Error = ParseSemVerError;
    fn try_from(value: &str) -> Result<SemVer, Self::Error> {
        let tokens: Vec<&str> = value.split('.').collect();
        if tokens.len() != 3 {
            return Err(ParseSemVerError::InvalidVersionFormat);
        }

        Ok(SemVer {
            major: tokens[0].parse()?,
            minor: tokens[1].parse()?,
            patch: tokens[2].parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryInto;

    #[test]
    fn should_compare_semver_versions() {
        assert!(SemVer::new(0, 0, 0) < SemVer::new(1, 2, 3));
        assert!(SemVer::new(1, 1, 0) < SemVer::new(1, 2, 0));
        assert!(SemVer::new(1, 0, 0) < SemVer::new(1, 2, 0));
        assert!(SemVer::new(1, 0, 0) < SemVer::new(1, 2, 3));
        assert!(SemVer::new(1, 2, 0) < SemVer::new(1, 2, 3));
        assert!(SemVer::new(1, 2, 3) == SemVer::new(1, 2, 3));
        assert!(SemVer::new(1, 2, 3) >= SemVer::new(1, 2, 3));
        assert!(SemVer::new(1, 2, 3) <= SemVer::new(1, 2, 3));
        assert!(SemVer::new(2, 0, 0) >= SemVer::new(1, 99, 99));
        assert!(SemVer::new(2, 0, 0) > SemVer::new(1, 99, 99));
    }

    #[test]
    fn parse_from_string() {
        let ver1: SemVer = "100.20.3".try_into().expect("should parse");
        assert_eq!(ver1, SemVer::new(100, 20, 3));
        let ver2: SemVer = "0.0.1".try_into().expect("should parse");
        assert_eq!(ver2, SemVer::new(0, 0, 1));

        assert!(SemVer::try_from("1.a.2.3").is_err());
        assert!(SemVer::try_from("1. 2.3").is_err());
        assert!(SemVer::try_from("12345124361461.0.1").is_err());
        assert!(SemVer::try_from("1.2.3.4").is_err());
        assert!(SemVer::try_from("1.2").is_err());
        assert!(SemVer::try_from("1").is_err());
        assert!(SemVer::try_from("0").is_err());
    }
}
