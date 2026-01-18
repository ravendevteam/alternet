//! This crate provides a unified network behaviour for a libp2p-based peer-to-peer application.
//! 
//! It aggregates multiple protocols into a single `Swarm` and exposes a combined `Event` type
//! that represents events from all supported protocols.
//! 
//! The supported protocols include:
//! - `Identify`: Protocol for exchanging node identity information.
//! - `Ping`: Simple round-trip time measurement between peers.
//! - `Kademlia (Kad)`: Distributed hash table for peer and content discovery.
//! - `Gossipsub`: PubSub protocol for message broadcasting.
//! - `Relay`: Support for relayed connections through other peers.
//! - `DCUtR`: Direct Connection Upgrade Through Relay for NAT traversal.
//! - `mDNS`: Local network peer discovery.

modwire::expose!(pub framework);

pub type Swarm = swarm::Swarm<Behaviour>;
pub type SwarmEvent = swarm::SwarmEvent<Event>;

#[allow(clippy::large_enum_variant)] // we can optimize this a lot later.
#[derive(derive_more::From)]
#[from(identify::Event)]
#[from(ping::Event)]
#[from(kad::Event)]
#[from(gossipsub::Event)]
#[from(relay::Event)]
#[from(dcutr::Event)]
#[from(mdns::Event)]
pub enum Event {
    Identify(identify::Event),
    Ping(ping::Event),
    Kad(kad::Event),
    Gossipsub(gossipsub::Event),
    Relay(relay::Event),
    Dcutr(dcutr::Event),
    Mdns(mdns::Event)
}

#[derive(libp2p::swarm::NetworkBehaviour)]
#[behaviour(out_event = "Event")]
pub struct Behaviour {
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
    pub relay: relay::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub mdns: mdns::tokio::Behaviour
}