// TODO - remove once schemars stops causing warning.
#![allow(clippy::field_reassign_with_default)]

use alloc::{string::String, vec::Vec};

#[cfg(feature = "std")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::bytesrepr::{self, FromBytes, ToBytes};

/// A named key.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Default, Debug)]
#[cfg_attr(feature = "std", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct NamedKey {
    /// The name of the entry.
    pub name: String,
    /// The value of the entry: a casper `Key` type.
    pub key: String,
}

impl ToBytes for NamedKey {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), bytesrepr::Error> {
        self.name.to_bytes(sink)?;
        self.key.to_bytes(sink)
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.name.serialized_length() + self.key.serialized_length()
    }
}

impl FromBytes for NamedKey {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (name, remainder) = String::from_bytes(bytes)?;
        let (key, remainder) = String::from_bytes(remainder)?;
        let named_key = NamedKey { name, key };
        Ok((named_key, remainder))
    }
}
