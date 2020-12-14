//! This module is home to the libp2p `GossipSub` behavior, used for gossiping this node's listening
//! addresses in order to allow peers to discover and connect to it.

use std::{
    error::Error as StdError,
    io,
    task::{Context, Poll},
};

use libp2p::{
    core::{
        connection::{ConnectionId, ListenerId},
        ConnectedPoint, ProtocolName,
    },
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubEvent, MessageAuthenticity, Topic,
        ValidationMode,
    },
    swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters, ProtocolsHandler},
    Multiaddr, PeerId,
};
use once_cell::sync::Lazy;
use tracing::{trace, warn};

use super::{Config, Error, Message, PayloadT, ProtocolId};
use crate::{components::chainspec_loader::Chainspec, types::NodeId};

/// The inner portion of the `ProtocolId` for the gossip behavior.  A standard prefix and suffix
/// will be applied to create the full protocol name.
const PROTOCOL_NAME_INNER: &str = "validator/gossip";

static TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("all".into()));

pub(super) struct GossipMessage(pub Vec<u8>);

impl GossipMessage {
    pub(super) fn new<P: PayloadT>(message: &Message<P>, max_size: u32) -> Result<Self, Error> {
        let serialized_message =
            bincode::serialize(message).map_err(|error| Error::Serialization(*error))?;

        if serialized_message.len() > max_size as usize {
            return Err(Error::MessageTooLarge {
                max_size,
                actual_size: serialized_message.len() as u64,
            });
        }

        Ok(GossipMessage(serialized_message))
    }
}

impl From<GossipMessage> for Vec<u8> {
    fn from(message: GossipMessage) -> Self {
        message.0
    }
}

/// Implementor of the libp2p `NetworkBehaviour` for gossiping.
pub(in crate::components::network) struct Behavior {
    gossipsub: Gossipsub,
    our_id: NodeId,
}

impl Behavior {
    pub(in crate::components::network) fn new(
        config: &Config,
        chainspec: &Chainspec,
        our_id: NodeId,
    ) -> Self {
        let protocol_id = ProtocolId::new(chainspec, PROTOCOL_NAME_INNER);
        let gossipsub_config = GossipsubConfigBuilder::new()
            .protocol_id(protocol_id.protocol_name().to_vec())
            .heartbeat_interval(config.gossip_heartbeat_interval.into())
            .max_transmit_size(config.gossip_max_message_size as usize)
            .duplicate_cache_time(config.gossip_duplicate_cache_timeout.into())
            .validation_mode(ValidationMode::Permissive)
            .build();
        let our_peer_id = match &our_id {
            NodeId::P2p(peer_id) => peer_id.clone(),
            _ => unreachable!(),
        };
        let mut gossipsub =
            Gossipsub::new(MessageAuthenticity::Author(our_peer_id), gossipsub_config);
        gossipsub.subscribe(TOPIC.clone());
        Behavior { gossipsub, our_id }
    }

    /// Gossips the given message.
    pub(in crate::components::network) fn gossip_message<P: PayloadT>(
        &mut self,
        message: GossipMessage,
    ) {
        if let Err(error) = self.gossipsub.publish(&*TOPIC, message) {
            warn!(?error, "{}: failed to gossip message", self.our_id);
        }
    }

    /// Called when `self.gossipsub` generates an event.
    ///
    /// Returns a `GossipMessage` if the event provided one.
    fn handle_generated_event(&mut self, event: GossipsubEvent) -> Option<GossipMessage> {
        match event {
            GossipsubEvent::Message(received_from, _, message) => {
                trace!(?message, "{}: received message via gossip", self.our_id);

                let source = match &message.source {
                    Some(peer_id) => peer_id.clone(),
                    None => {
                        warn!(
                            ?message,
                            "{}: received gossiped message with no source ID", self.our_id
                        );
                        return None;
                    }
                };

                return Some(GossipMessage(message.data));
            }
            GossipsubEvent::Subscribed { peer_id, topic } => {
                trace!(%peer_id, %topic, "{}: peer subscribed to gossip topic", self.our_id)
            }
            GossipsubEvent::Unsubscribed { peer_id, topic } => {
                trace!(%peer_id, %topic, "{}: peer unsubscribed from gossip topic", self.our_id)
            }
        }
        None
    }
}

impl NetworkBehaviour for Behavior {
    type ProtocolsHandler = <Gossipsub as NetworkBehaviour>::ProtocolsHandler;
    type OutEvent = GossipMessage;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        self.gossipsub.new_handler()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.gossipsub.addresses_of_peer(peer_id)
    }

    fn inject_connected(&mut self, peer_id: &PeerId) {
        self.gossipsub.inject_connected(peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        self.gossipsub.inject_disconnected(peer_id);
    }

    fn inject_connection_established(
        &mut self,
        peer_id: &PeerId,
        connection_id: &ConnectionId,
        endpoint: &ConnectedPoint,
    ) {
        self.gossipsub
            .inject_connection_established(peer_id, connection_id, endpoint);
    }

    fn inject_address_change(
        &mut self,
        peer_id: &PeerId,
        connection_id: &ConnectionId,
        old: &ConnectedPoint,
        new: &ConnectedPoint,
    ) {
        self.gossipsub
            .inject_address_change(peer_id, connection_id, old, new);
    }

    fn inject_connection_closed(
        &mut self,
        peer_id: &PeerId,
        connection_id: &ConnectionId,
        endpoint: &ConnectedPoint,
    ) {
        self.gossipsub
            .inject_connection_closed(peer_id, connection_id, endpoint);
    }

    fn inject_addr_reach_failure(
        &mut self,
        peer_id: Option<&PeerId>,
        addr: &Multiaddr,
        error: &dyn StdError,
    ) {
        self.gossipsub
            .inject_addr_reach_failure(peer_id, addr, error);
    }

    fn inject_dial_failure(&mut self, peer_id: &PeerId) {
        self.gossipsub.inject_dial_failure(peer_id);
    }

    fn inject_new_listen_addr(&mut self, addr: &Multiaddr) {
        self.gossipsub.inject_new_listen_addr(addr);
    }

    fn inject_expired_listen_addr(&mut self, addr: &Multiaddr) {
        self.gossipsub.inject_expired_listen_addr(addr);
    }

    fn inject_new_external_addr(&mut self, addr: &Multiaddr) {
        self.gossipsub.inject_new_external_addr(addr);
    }

    fn inject_listener_error(&mut self, id: ListenerId, err: &(dyn StdError + 'static)) {
        self.gossipsub.inject_listener_error(id, err);
    }

    fn inject_listener_closed(&mut self, id: ListenerId, reason: Result<(), &io::Error>) {
        self.gossipsub.inject_listener_closed(id, reason);
    }

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent,
    ) {
        self.gossipsub.inject_event(peer_id, connection_id, event);
    }

    fn poll(
        &mut self,
        context: &mut Context,
        poll_params: &mut impl PollParameters,
    ) -> Poll<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
            Self::OutEvent,
        >,
    > {
        // Simply pass most action variants though.  We're only interested in the `GeneratedEvent`
        // variant.  These can be all be handled without needing to return `Poll::Ready` until we
        // get an incoming message event.
        loop {
            match self.gossipsub.poll(context, poll_params) {
                Poll::Ready(NetworkBehaviourAction::GenerateEvent(event)) => {
                    if let Some(gossip_message[[) = self.handle_generated_event(event) {
                        return Poll::Ready(NetworkBehaviourAction::GenerateEvent(gossip_message));
                    }
                }
                Poll::Ready(NetworkBehaviourAction::DialAddress { address }) => {
                    warn!(%address, "should not dial address via addresses-gossiper behavior");
                    return Poll::Ready(NetworkBehaviourAction::DialAddress { address });
                }
                Poll::Ready(NetworkBehaviourAction::DialPeer { peer_id, condition }) => {
                    warn!(%peer_id, "should not dial peer via addresses-gossiper behavior");
                    return Poll::Ready(NetworkBehaviourAction::DialPeer { peer_id, condition });
                }
                Poll::Ready(NetworkBehaviourAction::NotifyHandler {
                    peer_id,
                    handler,
                    event,
                }) => {
                    return Poll::Ready(NetworkBehaviourAction::NotifyHandler {
                        peer_id,
                        handler,
                        event,
                    });
                }
                Poll::Ready(NetworkBehaviourAction::ReportObservedAddr { address }) => {
                    return Poll::Ready(NetworkBehaviourAction::ReportObservedAddr { address });
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
