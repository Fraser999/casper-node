use std::{
    error::Error as StdError,
    io, iter,
    marker::PhantomData,
    task::{Context, Poll},
};

use libp2p::{
    core::{
        connection::{ConnectionId, ListenerId},
        ConnectedPoint,
    },
    request_response::{
        ProtocolSupport, RequestResponse, RequestResponseConfig, RequestResponseEvent,
        RequestResponseMessage,
    },
    swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters, ProtocolsHandler},
    Multiaddr, PeerId,
};
use tracing::{trace, warn};

use super::{Codec, Incoming, Outgoing};
use crate::{
    components::{
        chainspec_loader::Chainspec,
        network::{Config, Message, PayloadT, ProtocolId},
    },
    types::NodeId,
};

/// The inner portion of the `ProtocolId` for the one-way message behavior.  A standard prefix and
/// suffix will be applied to create the full protocol name.
const PROTOCOL_NAME_INNER: &str = "validator/one-way";

/// Implementor of the libp2p `NetworkBehaviour` for one-way messages.
///
/// This is a wrapper round a `RequestResponse` where the response type is defined to be the unit
/// value.
pub(in crate::components::network) struct Behavior {
    libp2p_req_resp: RequestResponse<Codec>,
    our_id: NodeId,
}

impl Behavior {
    pub(in crate::components::network) fn new(
        config: &Config,
        chainspec: &Chainspec,
        our_id: NodeId,
    ) -> Self {
        let codec = Codec::from(config);
        let protocol_id = ProtocolId::new(chainspec, PROTOCOL_NAME_INNER);
        let request_response_config = RequestResponseConfig::from(config);
        let libp2p_req_resp = RequestResponse::new(
            codec,
            iter::once((protocol_id, ProtocolSupport::Full)),
            request_response_config,
        );
        Behavior {
            libp2p_req_resp,
            our_id,
        }
    }

    /// Sends a one-way message to a peer.
    pub(in crate::components::network) fn send_message(&mut self, outgoing_message: Outgoing) {
        let request_id = self
            .libp2p_req_resp
            .send_request(destination, outgoing_message);
        trace!("{}: sent one-way message {}", self.our_id, request_id);
    }

    /// Called when `self.libp2p_req_resp` generates an event.
    ///
    /// The only event type which will cause the return to be `Some` is an incoming request.  All
    /// other variants simply involve generating a log message.
    fn handle_generated_event(
        &mut self,
        event: RequestResponseEvent<Vec<u8>, ()>,
    ) -> Option<Incoming> {
        trace!("{}: {:?}", self.our_id, event);

        match event {
            RequestResponseEvent::Message {
                peer,
                message: RequestResponseMessage::Request { request, .. },
            } => {
                return Some(Incoming::new(peer, message));
            }
            RequestResponseEvent::Message {
                message: RequestResponseMessage::Response { .. },
                ..
            } => {
                // Note that a response will still be emitted immediately after the request has been
                // sent, since `RequestResponseCodec::read_response` for the one-way Codec does not
                // actually read anything from the given I/O stream.
            }
            RequestResponseEvent::OutboundFailure {
                peer,
                request_id,
                error,
            } => {
                warn!(
                    ?peer,
                    ?request_id,
                    ?error,
                    "{}: outbound failure",
                    self.our_id
                )
            }
            RequestResponseEvent::InboundFailure {
                peer,
                request_id,
                error,
            } => {
                warn!(
                    ?peer,
                    ?request_id,
                    ?error,
                    "{}: inbound failure",
                    self.our_id
                )
            }
        }

        None
    }
}

impl NetworkBehaviour for Behavior {
    type ProtocolsHandler = <RequestResponse<Codec> as NetworkBehaviour>::ProtocolsHandler;
    type OutEvent = Incoming;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        self.libp2p_req_resp.new_handler()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.libp2p_req_resp.addresses_of_peer(peer_id)
    }

    fn inject_connected(&mut self, peer_id: &PeerId) {
        self.libp2p_req_resp.inject_connected(peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        self.libp2p_req_resp.inject_disconnected(peer_id);
    }

    fn inject_connection_established(
        &mut self,
        peer_id: &PeerId,
        connection_id: &ConnectionId,
        endpoint: &ConnectedPoint,
    ) {
        self.libp2p_req_resp
            .inject_connection_established(peer_id, connection_id, endpoint);
    }

    fn inject_address_change(
        &mut self,
        peer_id: &PeerId,
        connection_id: &ConnectionId,
        old: &ConnectedPoint,
        new: &ConnectedPoint,
    ) {
        self.libp2p_req_resp
            .inject_address_change(peer_id, connection_id, old, new);
    }

    fn inject_connection_closed(
        &mut self,
        peer_id: &PeerId,
        connection_id: &ConnectionId,
        endpoint: &ConnectedPoint,
    ) {
        self.libp2p_req_resp
            .inject_connection_closed(peer_id, connection_id, endpoint);
    }

    fn inject_addr_reach_failure(
        &mut self,
        peer_id: Option<&PeerId>,
        addr: &Multiaddr,
        error: &dyn StdError,
    ) {
        self.libp2p_req_resp
            .inject_addr_reach_failure(peer_id, addr, error);
    }

    fn inject_dial_failure(&mut self, peer_id: &PeerId) {
        self.libp2p_req_resp.inject_dial_failure(peer_id);
    }

    fn inject_new_listen_addr(&mut self, addr: &Multiaddr) {
        self.libp2p_req_resp.inject_new_listen_addr(addr);
    }

    fn inject_expired_listen_addr(&mut self, addr: &Multiaddr) {
        self.libp2p_req_resp.inject_expired_listen_addr(addr);
    }

    fn inject_new_external_addr(&mut self, addr: &Multiaddr) {
        self.libp2p_req_resp.inject_new_external_addr(addr);
    }

    fn inject_listener_error(&mut self, id: ListenerId, err: &(dyn StdError + 'static)) {
        self.libp2p_req_resp.inject_listener_error(id, err);
    }

    fn inject_listener_closed(&mut self, id: ListenerId, reason: Result<(), &io::Error>) {
        self.libp2p_req_resp.inject_listener_closed(id, reason);
    }

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent,
    ) {
        self.libp2p_req_resp
            .inject_event(peer_id, connection_id, event);
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
            match self.libp2p_req_resp.poll(context, poll_params) {
                Poll::Ready(NetworkBehaviourAction::GenerateEvent(event)) => {
                    if let Some(incoming_message) = self.handle_generated_event(event) {
                        return Poll::Ready(NetworkBehaviourAction::GenerateEvent(
                            incoming_message,
                        ));
                    }
                }
                Poll::Ready(NetworkBehaviourAction::DialAddress { address }) => {
                    warn!(%address, "should not dial address via one-way message behavior");
                    return Poll::Ready(NetworkBehaviourAction::DialAddress { address });
                }
                Poll::Ready(NetworkBehaviourAction::DialPeer { peer_id, condition }) => {
                    warn!(%peer_id, "should not dial peer via one-way message behavior");
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
