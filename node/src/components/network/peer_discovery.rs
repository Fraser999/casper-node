//! This module is home to the libp2p `Kademlia` behavior, used for peer discovery.

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
    kad::{
        record::store::{MemoryStore, MemoryStoreConfig},
        Kademlia, KademliaConfig,
    },
    swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters, ProtocolsHandler},
    Multiaddr, PeerId,
};
use once_cell::sync::Lazy;
use semver::Version;
use tracing::{debug, trace, warn};

use super::{Config, ProtocolId};
use crate::{components::chainspec_loader::Chainspec, types::NodeId};

/// The inner portion of the `ProtocolId` for the peer-discovery message behavior.  A standard
/// prefix and suffix will be applied to create the full protocol name.
const PROTOCOL_NAME_INNER: &str = "peer-discovery";

/// Implementor of the libp2p `NetworkBehaviour` for peer discovery via Kademlia lookups.
pub(super) struct Behavior {
    kademlia: Kademlia<MemoryStore>,
    our_id: NodeId,
}

impl Behavior {
    pub(super) fn new(config: &Config, chainspec: &Chainspec, our_id: NodeId) -> Self {
        let our_peer_id = match &our_id {
            NodeId::P2p(peer_id) => peer_id.clone(),
            _ => unreachable!(),
        };

        // We don't intend to actually store anything in the Kademlia DHT, so configure accordingly.
        let memory_store_config = MemoryStoreConfig {
            max_records: 0,
            max_value_bytes: 0,
            ..Default::default()
        };
        let memory_store = MemoryStore::with_config(our_peer_id.clone(), memory_store_config);

        let protocol_id = ProtocolId::new(chainspec, PROTOCOL_NAME_INNER);
        let mut kademlia_config = KademliaConfig::default();
        kademlia_config
            .set_protocol_name(protocol_id.protocol_name().to_vec())
            // Require iterative queries to use disjoint paths for increased security.
            .disjoint_query_paths(true)
            // Closes the connection if it's idle for this amount of time.
            .set_connection_idle_timeout(config.connection_keep_alive.into());
        let kademlia = Kademlia::with_config(our_peer_id, memory_store, kademlia_config);

        Behavior { kademlia, our_id }
    }

    // We must explicitly call this once we've bootstrapped to at least one peer in order to join
    // the Kademlia overlay network.
    pub(super) fn add_peer(&mut self, peer: &PeerId, address: Multiaddr) {
        warn!("adding peer to kad: {}, {}", peer, address);
        let _ = self.kademlia.add_address(peer, address);
    }

    pub(super) fn random_lookup(&mut self) {
        // TODO - don't do lookup if we have "enough" peer connections (for some value of "enough").
        let random_address = PeerId::random();
        debug!(
            "{}: random kademlia lookup for peers closest to {:?}",
            self.our_id, random_address
        );
        self.kademlia.get_closest_peers(random_address);
    }
}

impl NetworkBehaviour for Behavior {
    type ProtocolsHandler = <Kademlia<MemoryStore> as NetworkBehaviour>::ProtocolsHandler;
    type OutEvent = ();

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        self.kademlia.new_handler()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.kademlia.addresses_of_peer(peer_id)
    }

    fn inject_connected(&mut self, peer_id: &PeerId) {
        self.kademlia.inject_connected(peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        self.kademlia.inject_disconnected(peer_id);
    }

    fn inject_connection_established(
        &mut self,
        peer_id: &PeerId,
        connection_id: &ConnectionId,
        endpoint: &ConnectedPoint,
    ) {
        self.kademlia
            .inject_connection_established(peer_id, connection_id, endpoint);
    }

    fn inject_address_change(
        &mut self,
        peer_id: &PeerId,
        connection_id: &ConnectionId,
        old: &ConnectedPoint,
        new: &ConnectedPoint,
    ) {
        self.kademlia
            .inject_address_change(peer_id, connection_id, old, new);
    }

    fn inject_connection_closed(
        &mut self,
        peer_id: &PeerId,
        connection_id: &ConnectionId,
        endpoint: &ConnectedPoint,
    ) {
        self.kademlia
            .inject_connection_closed(peer_id, connection_id, endpoint);
    }

    fn inject_addr_reach_failure(
        &mut self,
        peer_id: Option<&PeerId>,
        addr: &Multiaddr,
        error: &dyn StdError,
    ) {
        self.kademlia
            .inject_addr_reach_failure(peer_id, addr, error);
    }

    fn inject_dial_failure(&mut self, peer_id: &PeerId) {
        self.kademlia.inject_dial_failure(peer_id);
    }

    fn inject_new_listen_addr(&mut self, addr: &Multiaddr) {
        self.kademlia.inject_new_listen_addr(addr);
    }

    fn inject_expired_listen_addr(&mut self, addr: &Multiaddr) {
        self.kademlia.inject_expired_listen_addr(addr);
    }

    fn inject_new_external_addr(&mut self, addr: &Multiaddr) {
        self.kademlia.inject_new_external_addr(addr);
    }

    fn inject_listener_error(&mut self, id: ListenerId, err: &(dyn StdError + 'static)) {
        self.kademlia.inject_listener_error(id, err);
    }

    fn inject_listener_closed(&mut self, id: ListenerId, reason: Result<(), &io::Error>) {
        self.kademlia.inject_listener_closed(id, reason);
    }

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent,
    ) {
        self.kademlia.inject_event(peer_id, connection_id, event);
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
            match self.kademlia.poll(context, poll_params) {
                Poll::Ready(NetworkBehaviourAction::GenerateEvent(event)) => {
                    warn!("{:?}", event);
                    // return Poll::Ready(NetworkBehaviourAction::GenerateEvent(()));
                }
                Poll::Ready(NetworkBehaviourAction::DialAddress { address }) => {
                    return Poll::Ready(NetworkBehaviourAction::DialAddress { address });
                }
                Poll::Ready(NetworkBehaviourAction::DialPeer { peer_id, condition }) => {
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
