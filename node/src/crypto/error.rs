use std::result;

use hex::FromHexError;
use pem::PemError;
use thiserror::Error;

use crate::utils::{ReadFileError, WriteFileError};
use casper_types::crypto;

/// A specialized `std::result::Result` type for cryptographic errors.
pub type Result<T> = result::Result<T, Error>;

/// Cryptographic errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Error resulting from creating or using asymmetric key types.
    #[error("asymmetric key error: {0}")]
    AsymmetricKey(crypto::Error),

    /// Error resulting when decoding a type from a hex-encoded representation.
    #[error("parsing from hex: {0}")]
    FromHex(#[from] FromHexError),

    /// Error trying to read a secret key.
    #[error("secret key load failed: {0}")]
    SecretKeyLoad(ReadFileError),

    /// Error trying to read a public key.
    #[error("public key load failed: {0}")]
    PublicKeyLoad(ReadFileError),

    /// Pem format error.
    #[error("pem error: {0}")]
    FromPem(String),

    /// DER format error.
    #[error("der error: {0}")]
    FromDer(#[from] derp::Error),

    /// DER format error - invalid tag provided.
    #[error("der error: invalid tag")]
    FromDerInvalidTag,

    /// Error trying to write a secret key.
    #[error("secret key save failed: {0}")]
    SecretKeySave(WriteFileError),

    /// Error trying to write a public key.
    #[error("public key save failed: {0}")]
    PublicKeySave(WriteFileError),

    /// Error trying to manipulate the system key.
    #[error("invalid operation on system key: {0}")]
    System(String),

    /// Error in getting random bytes from the system's preferred random number source.
    #[error("failed to get random bytes: {0}")]
    GetRandomBytes(#[from] getrandom::Error),

    /// Failed to verify an Ed25519 signature.
    #[error("failed to verify ed25519 signature")]
    Ed25519FailedToVerify,

    /// Failed to verify a Secp256k1 signature.
    #[error("failed to verify secp256k1 signature")]
    Secp256k1FailedToVerify,

    /// Mismatch between type of PublicKey and type of Signature.
    #[error("mismatch between public key and signature type")]
    PublicKeyVsSignatureMismatch,
}

impl From<PemError> for Error {
    fn from(error: PemError) -> Self {
        Error::FromPem(error.to_string())
    }
}

impl From<crypto::Error> for Error {
    fn from(error: crypto::Error) -> Self {
        Error::AsymmetricKey(error)
    }
}
