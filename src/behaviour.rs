use crate::control;

use ::parking_lot::Mutex;
use ::std::collections::HashMap;
use ::std::sync::Arc;

use crate::prelude::*;
use multiaddr::{Multiaddr, PeerId};

pub struct Behaviour {
    control_receiver: control::RequestReceiver,
    control: control::Control,
    /// map of all ongoing lookups on the kad-dht and the return channel
    lookups: Arc<Mutex<HashMap<kad::QueryId, control::resolve::ResponseSender>>>,
    delegating: delegating::Behaviour,
}

impl Behaviour {
    pub fn new_control(&self) -> control::Control {
        self.control.clone()
    }
}

mod delegating {
    use ::libp2p::*;
    
    #[allow(unused)]
    use swarm::behaviour::toggle::Toggle;

    #[derive(::libp2p::swarm::NetworkBehaviour)]
    pub struct Behaviour {
        identify: identify::Behaviour,
        pub(crate) kad: ::libp2p::kad::Behaviour<kad::store::MemoryStore>,
        relay: relay::client::Behaviour,
        #[cfg(feature = "autonat")]
        autonat: Toggle<autonat::Behaviour>,
        #[cfg(feature = "dcutr")]
        dcutr: Toggle<dcutr::Behaviour>,
        #[cfg(feature = "mdns")]
        mdns: Toggle<mdns::tokio::Behaviour>,
    }
}

impl swarm::NetworkBehaviour for Behaviour {
    type ConnectionHandler = <delegating::Behaviour as swarm::NetworkBehaviour>::ConnectionHandler;
    type ToSwarm = <delegating::Behaviour as swarm::NetworkBehaviour>::ToSwarm;

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: swarm::ConnectionId,
        peer: PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<swarm::THandler<Self>, swarm::ConnectionDenied> {
        self.delegating.handle_established_inbound_connection(
            connection_id,
            peer,
            local_addr,
            remote_addr,
        )
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: swarm::ConnectionId,
        peer: PeerId,
        addr: &Multiaddr,
        role_override: core::Endpoint,
        port_use: core::transport::PortUse,
    ) -> Result<swarm::THandler<Self>, swarm::ConnectionDenied> {
        self.delegating.handle_established_outbound_connection(
            connection_id,
            peer,
            addr,
            role_override,
            port_use,
        )
    }

    fn on_swarm_event(&mut self, event: swarm::FromSwarm) {
        self.delegating.on_swarm_event(event);
        // todo
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: swarm::ConnectionId,
        event: swarm::THandlerOutEvent<Self>,
    ) {
        self.delegating
            .on_connection_handler_event(peer_id, connection_id, event);
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<swarm::ToSwarm<Self::ToSwarm, swarm::THandlerInEvent<Self>>> {
        use std::task::Poll;
        if let Poll::Ready(Some(command)) = self.control_receiver.poll_next_unpin(cx) {
            match command {
                control::Request::Resolve(request) => {
                    let query_id = self
                        .delegating
                        .kad
                        .get_record(libp2p::kad::RecordKey::new(&request.domain));
                    self.lookups
                        .lock()
                        .insert(query_id, request.responder)
                        .map(|_| {
                            // todo: tracing::error instead
                            panic!("ids wrapped - how the fuck did you do that???")
                        });
                }
                control::Request::Register(request) => todo!(),
                control::Request::Deregister(request) => todo!(),
            }
        }
        if let Poll::Ready(event) = self.delegating.poll(cx) {
            // todo: should this ever return?
            // i have the feeling it shouldn't, we don't really want to leak any events?
            return Poll::Ready(event);
        }
        Poll::Pending
    }
}
