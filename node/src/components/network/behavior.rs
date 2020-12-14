use derive_more::From;
use libp2p::{
    ping::{Ping, PingConfig, PingEvent},
    Multiaddr, NetworkBehaviour, PeerId,
};

use super::{
    Config, GossipBehavior, GossipMessage, OneWayIncomingMessage, OneWayMessageBehavior,
    OneWayOutgoingMessage, PayloadT, PeerDiscoveryBehavior,
};
use crate::{components::chainspec_loader::Chainspec, types::NodeId};

/// An enum defining the top-level events passed to the swarm's handler.  This will be received in
/// the swarm's handler wrapped in a `SwarmEvent::Behaviour`.
#[derive(Debug, From)]
pub(super) enum SwarmBehaviorEvent {
    OneWayMessage(OneWayIncomingMessage),
    #[from(ignore)]
    Discovery,
    Gossiper(Vec<u8>),
}

impl From<()> for SwarmBehaviorEvent {
    fn from(_: ()) -> Self {
        SwarmBehaviorEvent::Discovery
    }
}

/// The top-level behavior used in the libp2p swarm.  It holds all subordinate behaviors required to
/// operate the network component.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "SwarmBehaviorEvent<P>", event_process = false)]
pub(super) struct Behavior<P: PayloadT> {
    one_way_message_behavior: OneWayMessageBehavior,
    peer_discovery: PeerDiscoveryBehavior,
    gossiper: GossipBehavior,
}

impl<P: PayloadT> Behavior<P> {
    pub(super) fn new(config: &Config, chainspec: &Chainspec, our_id: NodeId) -> Self {
        let one_way_message_behavior =
            OneWayMessageBehavior::new(config, chainspec, our_id.clone());
        let peer_discovery = PeerDiscoveryBehavior::new(config, chainspec, our_id.clone());
        let gossiper = GossipBehavior::new(config, chainspec, our_id);
        Behavior {
            one_way_message_behavior,
            peer_discovery,
            gossiper,
        }
    }

    pub(super) fn send_one_way_message(&mut self, outgoing_message: OneWayOutgoingMessage) {
        self.one_way_message_behavior.send_message(outgoing_message);
    }

    pub(super) fn add_known_peer(&mut self, peer: &PeerId, address: Multiaddr) {
        self.peer_discovery.add_peer(peer, address)
    }

    pub(super) fn discover_peers(&mut self) {
        self.peer_discovery.random_lookup();
    }

    pub(super) fn gossip(&mut self, message: GossipMessage) {
        self.gossiper.publish(message);
    }
}
