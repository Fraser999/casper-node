use core::fmt::{self, Debug, Display, Formatter};

use base64::DecodeError;
use hex::FromHexError; // Re-exported of signature::Error; used by both dalek and k256 libs

/// Cryptographic errors.
#[derive(Debug)]
pub enum Error {
    /// Error resulting when decoding a type from a hex-encoded representation.
    FromHex(FromHexError),
    FromHexNoTag,
    FromHexInvalidTag {
        provided_tag: u8,
    },

    /// Error resulting when decoding a type from a base64 representation.
    FromBase64(DecodeError),

    Ed25519SecretKeyFromBytes,
    Ed25519PublicKeyFromBytes {
        provided_bytes: Vec<u8>,
    },
    Ed25519SignatureFromBytes {
        provided_bytes: Vec<u8>,
    },
    Secp256k1SecretKeyFromBytes,
    Secp256k1PublicKeyFromBytes {
        provided_bytes: Vec<u8>,
    },
    Secp256k1SignatureFromBytes {
        provided_bytes: Vec<u8>,
    },
}

impl Display for Error {}

impl From<FromHexError> for Error {
    fn from(error: FromHexError) -> Self {
        Error::FromHex(error)
    }
}
