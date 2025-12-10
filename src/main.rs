mod p2p {
    pub use ::libp2p::PeerId;
    pub use ::libp2p::SwarmBuilder;
    pub use ::libp2p::core::transport::Boxed;
    pub use ::libp2p::identity::Keypair as IdentityKeypair;
    pub use ::libp2p::identity::PublicKey as IdentityPublicKey;
    pub use ::libp2p::multiaddr::Protocol;
    pub use ::libp2p::multiaddr::Multiaddr as Address;
    pub use ::libp2p::noise::Config as NoiseConfig;
    pub use ::libp2p::quic::Config as QuicConfig;
    pub use ::libp2p::swarm::Swarm;
    pub use ::libp2p::swarm::SwarmEvent;
    pub use ::libp2p::tcp;
    pub use ::libp2p::yamux::Config as YamuxConfig;
    pub use ::libp2p::kad::store::RecordStore;
    pub use ::libp2p::kad::RecordKey;
}

#[tokio::main]
async fn main() {
    let _ = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            Default::default(),
            (libp2p::tls::Config::new, libp2p::noise::Config::new),
            libp2p::yamux::Config::default,
        )
        .unwrap()
        .with_quic_config(|config| config)
        .with_behaviour(|_| libp2p::swarm::dummy::Behaviour)
        .unwrap()
        .build();
}

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


// mesh network??
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