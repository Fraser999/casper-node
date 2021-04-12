use alloc::vec::Vec;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{
    bytesrepr::{Error, FromBytes, ToBytes},
    CLType, CLTyped,
};

/// The number of bytes in a serialized [`Phase`].
pub const PHASE_SERIALIZED_LENGTH: usize = 1;

/// The phase in which a given contract is executing.
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(u8)]
pub enum Phase {
    /// Set while committing the genesis or upgrade configurations.
    System = 0,
    /// Set while executing the payment code of a deploy.
    Payment = 1,
    /// Set while executing the session code of a deploy.
    Session = 2,
    /// Set while finalizing payment at the end of a deploy.
    FinalizePayment = 3,
}

impl ToBytes for Phase {
    #[inline(always)]
    fn to_bytes(&self, sink: &mut Vec<u8>) -> Result<(), Error> {
        sink.push(*self as u8);
        Ok(())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        PHASE_SERIALIZED_LENGTH
    }
}

impl FromBytes for Phase {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (id, rest) = u8::from_bytes(bytes)?;
        let phase = FromPrimitive::from_u8(id).ok_or(Error::Formatting)?;
        Ok((phase, rest))
    }
}

impl CLTyped for Phase {
    fn cl_type() -> CLType {
        CLType::U8
    }
}
