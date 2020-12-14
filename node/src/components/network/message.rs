use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message<P>(P);

impl<P> Message<P> {
    pub fn new(payload: P) -> Self {
        Message(payload)
    }

    pub fn into_payload(self) -> P {
        self.0
    }
}

impl<P: Display> Display for Message<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "payload: {}", self.0)
    }
}
