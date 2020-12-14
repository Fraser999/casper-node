//! This module is home to the infrastructure to support "one-way" messages, i.e. requests which
//! expect no response.
//!
//! For now, as a side-effect of the original small_network component, all peer-to-peer messages are
//! one-way.

mod behavior;
mod message;

pub(super) use behavior::Behavior;
pub(super) use message::{Codec, Incoming, Outgoing};
