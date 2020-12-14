use std::{
    fmt::{self, Debug, Display, Formatter},
    future::Future,
    io,
    pin::Pin,
};

use futures::{AsyncReadExt, AsyncWriteExt, FutureExt};
use futures_io::{AsyncRead, AsyncWrite};
use libp2p::{request_response::RequestResponseCodec, PeerId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    components::network::{Config, Error, Message, PayloadT, ProtocolId},
    types::NodeId,
};

#[derive(Debug)]
pub(in crate::components::network) struct Incoming {
    pub source: PeerId,
    pub message: Vec<u8>,
}

impl Incoming {}

#[derive(Debug)]
pub(in crate::components::network) struct Outgoing {
    pub destination: PeerId,
    pub message: Vec<u8>,
}

impl Outgoing {
    pub(in crate::components::network) fn new<P: PayloadT>(
        destination: NodeId,
        message: &Message<P>,
        max_size: u32,
    ) -> Result<Self, Error> {
        let serialized_message =
            bincode::serialize(message).map_err(|error| Error::Serialization(*error))?;

        if serialized_message.len() > max_size as usize {
            return Err(Error::MessageTooLarge {
                max_size,
                actual_size: serialized_message.len() as u64,
            });
        }

        match &destination {
            NodeId::P2p(destination) => Ok(Outgoing {
                destination: destination.clone(),
                message: serialized_message,
            }),
            destination => {
                unreachable!(
                    "can't send to {} (small_network node ID) via libp2p",
                    destination
                )
            }
        }
    }
}

impl From<Outgoing> for Vec<u8> {
    fn from(outgoing: Outgoing) -> Self {
        outgoing.message
    }
}

/// Implements libp2p `RequestResponseCodec` for one-way messages, i.e. requests which expect no
/// response.
#[derive(Debug, Clone)]
pub struct Codec {
    max_message_size: u32,
}

impl From<&Config> for Codec {
    fn from(config: &Config) -> Self {
        Codec {
            max_message_size: config.max_one_way_message_size,
        }
    }
}

impl RequestResponseCodec for Codec {
    type Protocol = ProtocolId;
    type Request = Vec<u8>;
    type Response = ();

    fn read_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
    ) -> Pin<Box<dyn Future<Output = io::Result<Self::Request>> + 'async_trait + Send>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
        T: AsyncRead + Unpin + Send + 'async_trait,
    {
        async move {
            // Read the length.
            let mut buffer = [0; 4];
            io.read(&mut buffer[..])
                .await
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
            let length = u32::from_le_bytes(buffer);
            if length > self.max_message_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "message size exceeds limit: {} > {}",
                        length, self.max_message_size
                    ),
                ));
            }

            // Read the payload.
            let mut buffer = vec![0; length as usize];
            io.read_exact(&mut buffer).await?;
            Ok(buffer)
        }
        .boxed()
    }

    fn read_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        _io: &'life2 mut T,
    ) -> Pin<Box<dyn Future<Output = io::Result<Self::Response>> + 'async_trait + Send>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
        T: AsyncRead + Unpin + Send + 'async_trait,
    {
        // For one-way messages, where no response will be sent by the peer, just return Ok(()).
        async { Ok(()) }.boxed()
    }

    fn write_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
        request: Self::Request,
    ) -> Pin<Box<dyn Future<Output = io::Result<()>> + 'async_trait + Send>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
        T: AsyncWrite + Unpin + Send + 'async_trait,
    {
        async move {
            // Write the length.
            if request.len() > self.max_message_size as usize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "message size exceeds limit: {} > {}",
                        request.len(),
                        self.max_message_size
                    ),
                ));
            }
            let length = request.len() as u32;
            io.write_all(&length.to_le_bytes()).await?;

            // Write the payload.
            io.write_all(&request).await?;

            io.close().await?;
            Ok(())
        }
        .boxed()
    }

    fn write_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        _io: &'life2 mut T,
        _response: Self::Response,
    ) -> Pin<Box<dyn Future<Output = io::Result<()>> + 'async_trait + Send>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
        T: AsyncWrite + Unpin + Send + 'async_trait,
    {
        // For one-way messages, where no response will be sent by the peer, just return Ok(()).
        async { Ok(()) }.boxed()
    }
}
