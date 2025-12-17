#![deny(clippy::unwrap_used)]

use ::std::sync;
use ::std::collections as coll;
use ::tokio::io;
use ::libp2p::futures;
use ::libp2p::identity;
use ::libp2p::quic;
use ::libp2p::ping;
use ::libp2p::kad;
use ::libp2p::kad::store as kad_store;
use ::libp2p::gossipsub;
use ::libp2p::request_response;
use ::libp2p::swarm;

use io::AsyncBufReadExt as _;
use futures::StreamExt as _;

::modwire::expose!(
    pub connection_event_handler
    pub domain
    pub event
    pub record
);

pub type Swarm = swarm::Swarm<Behaviour>;
pub type SwarmEvent = swarm::SwarmEvent<Event>;

#[::async_trait::async_trait]
pub trait EventHandlerExt {
    async fn handle(&mut self, swarm: &mut Swarm, event: &SwarmEvent);
}

#[derive(swarm::NetworkBehaviour)]
#[behaviour(out_event = "Event")]
pub struct Behaviour {
    pub ping: ping::Behaviour,
    pub kad: kad::Behaviour<kad_store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour
}

impl Behaviour {
    pub fn new(keypair: identity::Keypair, peer_id: ::libp2p::PeerId) -> Self {
        let ping_config: ping::Config = ping::Config::new();
        let ping: ping::Behaviour = ping::Behaviour::new(ping_config);
        let kad_store: kad_store::MemoryStore = kad_store::MemoryStore::new(peer_id);
        let kad: kad::Behaviour<kad_store::MemoryStore> = kad::Behaviour::new(peer_id, kad_store);
        let gossipsub_key: gossipsub::MessageAuthenticity = gossipsub::MessageAuthenticity::Signed(keypair);
        let gossipsub_config: gossipsub::Config = gossipsub::Config::default();
        let gossipsub: gossipsub::Behaviour = gossipsub::Behaviour::new(gossipsub_key, gossipsub_config).unwrap();
        Self {
            ping,
            kad,
            gossipsub
        }
    }
}

#[::tokio::main]
async fn main() {
    let keypair: identity::Keypair = identity::Keypair::generate_ed25519();
    let peer_id: ::libp2p::PeerId = keypair.public().into();
    let quic_config: quic::Config = quic::Config::new(&keypair);

    let mut swarm: ::libp2p::Swarm<_> = ::libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_behaviour(|keypair| Behaviour::new(keypair.to_owned(), peer_id))
        .unwrap()
        .build();

    let mut stdin: io::Lines<_> = ::tokio::io::BufReader::new(::tokio::io::stdin()).lines();

    // # Event Handler Registration
    //
    // ## Note
    //
    // - Handlers are executed sequentially; avoid long blocking operations.
    // - Do not call `swarm.select_next_some()` whilst handling events. Handler-polling happens in the main loop.
    let mut event_handlers: Vec<Box<dyn EventHandlerExt>> = vec![
        Box::new(ConnectionEventHandler)
    ];

    loop {
        ::tokio::select! {
            event = swarm.select_next_some() => {
                for event_handler in event_handlers.iter_mut() {
                    event_handler.handle(&mut swarm, &event).await;
                }
            },
            line = stdin.next_line() => match line {
                Ok(Some(command)) => println!("{}", command),
                Ok(None) => break,
                Err(e) => {
                    eprint!("{}", e);
                    break
                },
                _ => {}
            }
        }
    }
}

// networking stuff first
// dial relay as client
// lookup alternet site.com.. find dht ref to a website
// direct connection or through relay