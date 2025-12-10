use ::libp2p::futures::StreamExt as _;
use ::libp2p::identity;
use ::libp2p::quic;
use ::libp2p::ping;
use ::libp2p::kad;
use ::libp2p::kad::store as kad_store;
use ::libp2p::swarm;

mod network;

// How many nodes do we aim to support?
// What happens when nodes blackout? Data??
// What if there are two 

// # Topology

// unstructured

// n-dimensional grid
// 1-dimentional line
// 2-dimensional rectangle
// 3-dimensional box

// ## Tree
// 
// Hierarchical
// can increase number of children per peer and flatten

// ## Ring
// Used by kda
// All peers has a number, each node knows about its two neighbours
// 1 hop at a time routing can become slow if there's too many nodes
// vulnerable to network partitioning
//
//
// Hub and Spoke | Star Formation
// 
// 
// DHT
//
//
// Ring DHT Route Node > Serve Regular Node

// .Communication
// .Network Management Join & Discovery
// .. Leaving
// .. Detecting and Removing Failed Peers
// .DataStorage & Locality
// - User addr
// - Files
// - Streams
// - Key + Value pairs
// - Locating Data
// - Accessing Data - by 1 peer or N peers
// - Removing data
// - Detecting and removing unused, unreachable, or incomplete (broken) data  
//
// .Consensus
// How to agree on th given state (data) among a network of peers with no central authority?
// How to deal with network partitioning and consensus?
// Consensus algorithms?

// .Security
// How to avoid eaves-dropping?
// How to avoid peer identity impersonation?
// How to avoid fabrication?
// How to avoid replay attacks?
// How to provide peer anonymity?

// Topologies, communication protocol, and algorithms

// Sloppy hashing and self-organizing clusters - Michael J. Freedman and David Mazières (2002)
//
// Locality problems
//
//
//


// mesh network?
// forwarding 


// Tiger beatle style simulation
// Testing

//
//
// libp2p behaviours? ping? keep_alive?

// conf reader from json, environment... etc
//
// configuration and set up
//
// === main loop ===
// react to network events... 
//
// stdin command listener or handler

#[::tokio::main]
async fn main() {
    let key_pair: identity::Keypair = identity::Keypair::generate_ed25519();
    let peer_id: ::libp2p::PeerId = key_pair.public().into();
    let quic_config: quic::Config = quic::Config::new(&key_pair);
    let mut swarm: ::libp2p::Swarm<_> = ::libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_behaviour(|_| network::Network::new(peer_id))
        .expect("Swarm behaviour bind success")
        .build();
    loop {
        match swarm.select_next_some().await {
            swarm::SwarmEvent::Behaviour(network::Event::Kad(kad::Event::InboundRequest { request })) => {

            },
            swarm::SwarmEvent::Behaviour(network::Event::Kad(kad::Event::ModeChanged { new_mode })) => {

            },
            _ => {}
        }
    }
}