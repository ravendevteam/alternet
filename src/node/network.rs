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
    pub kad: kad::Behaviour<kad_store::MemoryStore>
}

impl Network {
    pub fn new(peer_id: ::libp2p::PeerId) -> Self {
        let ping_config: ping::Config = ping::Config::new();
        let ping: ping::Behaviour = ping::Behaviour::new(ping_config);
        let kad_store: kad_store::MemoryStore = kad_store::MemoryStore::new(peer_id);
        let kad: kad::Behaviour<kad_store::MemoryStore> = kad::Behaviour::new(peer_id, kad_store);
        Self {
            ping,
            kad
        }
    }
}