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
                        .get_record(libp2p::kad::RecordKey::new(&format!("addr:{}", request.domain.to_ascii())));
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
        let Poll::Ready(event) = self.delegating.poll(cx) else {
            return Poll::Pending;
        };
        let event = match event {
            swarm::ToSwarm::GenerateEvent(event) => event,
            // swarm::ToSwarm::Dial { opts } => todo!(),
            // swarm::ToSwarm::ListenOn { opts } => todo!(),
            // swarm::ToSwarm::RemoveListener { id } => todo!(),
            // swarm::ToSwarm::NotifyHandler { peer_id, handler, event } => todo!(),
            // swarm::ToSwarm::NewExternalAddrCandidate(multiaddr) => todo!(),
            // swarm::ToSwarm::ExternalAddrConfirmed(multiaddr) => todo!(),
            // swarm::ToSwarm::ExternalAddrExpired(multiaddr) => todo!(),
            // swarm::ToSwarm::CloseConnection { peer_id, connection } => todo!(),
            // swarm::ToSwarm::NewExternalAddrOfPeer { peer_id, address } => todo!(),
            _ => {
                return Poll::Pending;
            }
        };

        self.handle_behaviour_event(&event);

        // todo: should this ever return?
        // i have the feeling it shouldn't, we don't really want to leak any events?
        return Poll::Ready(swarm::ToSwarm::GenerateEvent(event));
    }
}

impl Behaviour {
    fn handle_behaviour_event(&mut self, event: &delegating::BehaviourEvent) {
        match &event {
            delegating::BehaviourEvent::Identify(identify) => todo!(),
            delegating::BehaviourEvent::Kad(kad) => match kad {
                kad::Event::InboundRequest { request } => todo!(),
                kad::Event::OutboundQueryProgressed {
                    id,
                    result,
                    stats,
                    step,
                } => match result {
                    kad::QueryResult::Bootstrap(bootstrap_ok) => todo!(),
                    kad::QueryResult::GetClosestPeers(get_closest_peers_ok) => todo!(),
                    kad::QueryResult::GetProviders(get_providers_ok) => todo!(),
                    kad::QueryResult::StartProviding(add_provider_ok) => todo!(),
                    kad::QueryResult::RepublishProvider(add_provider_ok) => todo!(),
                    kad::QueryResult::GetRecord(get_record_ok) => 'get_record: {
                        // todo: validate and best logic
                        // match get_record_ok {
                        //     Ok(ok) => match ok {
                        //         kad::GetRecordOk::FoundRecord(peer_record) => {
                        //             // self.delegating.kad.
                        //         }
                        //         kad::GetRecordOk::FinishedWithNoAdditionalRecord {
                        //             cache_candidates,
                        //         } => todo!(),
                        //     },
                        //     Err(_) => todo!(),
                        // }
                        let Some(result_sender) = step
                            .last
                            .then(|| {
                                let mut lookups_lock = self.lookups.lock();
                                lookups_lock.remove(id)
                            })
                            .flatten()
                        else {
                            break 'get_record;
                        };
                        // todo: send back
                    }
                    kad::QueryResult::PutRecord(put_record_ok) => todo!(),
                    kad::QueryResult::RepublishRecord(put_record_ok) => todo!(),
                },
                kad::Event::RoutingUpdated {
                    peer,
                    is_new_peer,
                    addresses,
                    bucket_range,
                    old_peer,
                } => todo!(),
                kad::Event::UnroutablePeer { peer } => todo!(),
                kad::Event::RoutablePeer { peer, address } => todo!(),
                kad::Event::PendingRoutablePeer { peer, address } => todo!(),
                kad::Event::ModeChanged { new_mode } => todo!(),
            },
            delegating::BehaviourEvent::Relay(relay) => todo!(),
            #[cfg(feature = "autonat")]
            delegating::BehaviourEvent::Autonat(autonat) => todo!(),
            #[cfg(feature = "dcutr")]
            delegating::BehaviourEvent::Dcutr(dcutr) => todo!(),
            #[cfg(feature = "mdns")]
            delegating::BehaviourEvent::Mdns(mdns) => match mdns {
                mdns::Event::Discovered(items) => {}
                mdns::Event::Expired(items) => todo!(),
            },
        }
    }
    fn validate_kad_record(&self, record: kad::Record) -> bool {
        if record.is_expired(std::time::Instant::now()) {
            return false;
        }
        let Some(peerid) = record.publisher else {
            return false;
        };

        todo!()
    }
    // fn add_
}
