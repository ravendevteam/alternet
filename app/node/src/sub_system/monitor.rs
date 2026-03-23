use super::*;

#[derive(Debug)]
pub struct Monitor;

impl SubSystem for Monitor {
    fn receive(
        &mut self, 
        swarm: &mut Swarm, 
        event: &mut Event, 
        queue: &mut dyn FnMut(Event)
    ) {
        match event.downcast_ref() {
            Some(SwarmEvent::ConnectionEstablished{
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in
            }) => {
                log::info!("connection established with {} via {:?}", peer_id, endpoint);
            },
            Some(SwarmEvent::ConnectionClosed{
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause
            }) => {
                log::info!("connection closed with {} via {:?}", peer_id, endpoint);
            },
            Some(SwarmEvent::Behaviour(BehaviourEvent::Autonat(autonat::Event::InboundProbe(autonat::InboundProbeEvent::Error{
                probe_id,
                peer,
                error
            })))) => {
                log::warn!("autonat inbound error [Id: {:?}]: failed to probe peer {}: {:?}", probe_id, peer, error);
            },
            Some(SwarmEvent::Behaviour(BehaviourEvent::Autonat(autonat::Event::InboundProbe(autonat::InboundProbeEvent::Request{
                probe_id,
                peer,
                addresses
            })))) => {
                log::info!("autonat inbound request [ID: {:?}]: peer {} requested probe for addresses: {:?}", probe_id, peer, addresses);
            },
            Some(SwarmEvent::Behaviour(BehaviourEvent::Autonat(autonat::Event::InboundProbe(autonat::InboundProbeEvent::Response{
                probe_id,
                peer,
                address
            })))) => {
                log::info!("autonat Inbound Response [ID: {:?}]: successfully probed peer {} at {}", probe_id, peer, address);
            },
            Some(SwarmEvent::Behaviour(BehaviourEvent::Autonat(autonat::Event::OutboundProbe(autonat::OutboundProbeEvent::Error{
                probe_id,
                peer,
                error
            })))) => {
                log::warn!("AutoNAT Outbound Error [ID: {:?}]: Bootstrap node {:?} could not reach us: {:?}", probe_id, peer, error);
            },
            Some(SwarmEvent::Behaviour(BehaviourEvent::Autonat(autonat::Event::OutboundProbe(autonat::OutboundProbeEvent::Request{
                probe_id,
                peer
            })))) => {
                log::info!("AutoNAT Outbound Request [ID: {:?}]: Asking bootstrap {:?} to probe our reachability", probe_id, peer);
            },
            Some(SwarmEvent::Behaviour(BehaviourEvent::Autonat(autonat::Event::OutboundProbe(autonat::OutboundProbeEvent::Response{
                probe_id,
                peer,
                address
            })))) => {
                log::info!("AutoNAT Outbound Response [ID: {:?}]: Bootstrap {} confirmed we are reachable at {}", probe_id, peer, address);
            },
            Some(SwarmEvent::Behaviour(BehaviourEvent::Autonat(autonat::Event::StatusChanged{
                old,
                new
            }))) => {
                log::info!("AutoNAT Status Change: {:?} -> {:?}", old, new);
                if matches!(new, autonat::NatStatus::Private) {
                    log::info!("Node is now PRIVATE. DCUtR hole punching is now eligible to trigger.");
                }
            },
            #[cfg(any(
                feature = "client", 
                feature = "server",
                feature = "malicious_client",
                feature = "malicious_server"
            ))]
            Some(SwarmEvent::Behaviour(BehaviourEvent::Dcutr(dcutr::Event{
                remote_peer_id,
                result
            }))) => {
                match result {
                    Ok(_) => {
                        log::info!("hole punch with peer {:?} succeeded", remote_peer_id);
                    },
                    Err(error) => {
                        log::warn!("hole punch with peer {:?} failed {:?}", remote_peer_id, error);
                    }
                }
            },
            #[cfg(any(
                feature = "client", 
                feature = "server",
                feature = "malicious_client",
                feature = "malicious_server"
            ))]
            Some(SwarmEvent::Behaviour(BehaviourEvent::RelayClient(relay::client::Event::InboundCircuitEstablished{
                src_peer_id,
                limit
            }))) => {
                log::info!("inbound circuit established with peer: {:?}, limit: {:?}", src_peer_id, limit);
            },
            #[cfg(any(
                feature = "client", 
                feature = "server",
                feature = "malicious_client",
                feature = "malicious_server"
            ))]
            Some(SwarmEvent::Behaviour(BehaviourEvent::RelayClient(relay::client::Event::OutboundCircuitEstablished{
                relay_peer_id,
                limit
            }))) => {
                log::info!("outbound circuit established with relay peer: {:?}, limit: {:?}", relay_peer_id, limit);
            },
            #[cfg(any(
                feature = "client", 
                feature = "server",
                feature = "malicious_client",
                feature = "malicious_server"
            ))]
            Some(SwarmEvent::Behaviour(BehaviourEvent::RelayClient(relay::client::Event::ReservationReqAccepted{
                relay_peer_id,
                renewal,
                limit
            }))) => {
                log::info!("reservation request accepted with relay peer: {:?}, renewal: {:?}, limit: {:?}", relay_peer_id, renewal, limit);
            },
            _ => {}
        }
    }
}