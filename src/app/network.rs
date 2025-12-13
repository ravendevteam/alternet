use std::collections::VecDeque;

use super::*;

#[allow(clippy::large_enum_variant)]
pub enum Event {
    Ping(ping::Event),
    Kad(kad::Event)
}

impl From<ping::Event> for Event {
    fn from(value: ping::Event) -> Self {
        Self::Ping(value)
    }
}

impl From<kad::Event> for Event {
    fn from(value: kad::Event) -> Self {
        Self::Kad(value)
    }
}

#[derive(swarm::NetworkBehaviour)]
#[behaviour(out_event = "Event")]
pub struct Network {
    pub ping: ping::Behaviour,
    pub kad: kad::Behaviour<kad_store::MemoryStore>,
    pub state: State
}

impl Network {
    pub fn new(peer_id: ::libp2p::PeerId) -> Self {
        let ping_config: ping::Config = ping::Config::new();
        let ping: ping::Behaviour = ping::Behaviour::new(ping_config);
        let kad_store: kad_store::MemoryStore = kad_store::MemoryStore::new(peer_id);
        let kad: kad::Behaviour<kad_store::MemoryStore> = kad::Behaviour::new(peer_id, kad_store);
        Self {
            ping,
            kad,
            state: State::new()
        }
    }
}





// implement smaller behaviours then add them to the main one

// behaviour 2

pub struct State {
    count: usize,
    events: VecDeque<()>
}

impl State {
    pub fn new() -> Self {
        State {
            count: 0,
            events: VecDeque::new(),
        }
    }
}

impl swarm::NetworkBehaviour for State {
    type ConnectionHandler = swarm::dummy::ConnectionHandler;
    type ToSwarm = super::Event;

    fn handle_established_inbound_connection(
        &mut self,
        _connection_id: swarm::ConnectionId,
        peer: libp2p::PeerId,
        local_addr: &libp2p::Multiaddr,
        remote_addr: &libp2p::Multiaddr,
    ) -> Result<swarm::THandler<Self>, swarm::ConnectionDenied> {
        Ok(swarm::dummy::ConnectionHandler)
    }

    fn handle_established_outbound_connection(
        &mut self,
        _connection_id: swarm::ConnectionId,
        peer: libp2p::PeerId,
        addr: &libp2p::Multiaddr,
        role_override: libp2p::core::Endpoint,
        port_use: libp2p::core::transport::PortUse,
    ) -> Result<swarm::THandler<Self>, swarm::ConnectionDenied> {
        Ok(swarm::dummy::ConnectionHandler)
    }

    fn on_swarm_event(&mut self, event: swarm::FromSwarm) {
        
    }

    fn on_connection_handler_event(
        &mut self,
        _peer_id: libp2p::PeerId,
        _connection_id: swarm::ConnectionId,
        _event: swarm::THandlerOutEvent<Self>,
    ) {
        
    }

    fn poll(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<swarm::ToSwarm<Self::ToSwarm, swarm::THandlerInEvent<Self>>> {
        ::std::task::Poll::Pending
    }
}