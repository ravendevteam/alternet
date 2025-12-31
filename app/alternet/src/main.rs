#![deny(clippy::unwrap_used)]

use libp2p::{futures, identity, quic, ping, kad, gossipsub, swarm};
use libp2p::kad::store as kad_store;
use tokio::io::{AsyncBufReadExt, BufReader};
use futures::StreamExt;

modwire::expose!(
    pub connection_event_handler
    pub domain
    pub event
    pub record
);

pub type Swarm = swarm::Swarm<Behaviour>;
pub type SwarmEvent = swarm::SwarmEvent<Event>;

#[async_trait::async_trait]
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
    pub fn new(keypair: identity::Keypair, peer_id: libp2p::PeerId) -> Self {
        let ping_config = ping::Config::new();
        let ping = ping::Behaviour::new(ping_config);
        
        let kad_store = kad_store::MemoryStore::new(peer_id);
        let kad = kad::Behaviour::new(peer_id, kad_store);
        
        let gossipsub_key = gossipsub::MessageAuthenticity::Signed(keypair);
        let gossipsub_config = gossipsub::Config::default();
        let gossipsub = gossipsub::Behaviour::new(gossipsub_key, gossipsub_config).unwrap();
        
        Self {
            ping,
            kad,
            gossipsub
        }
    }
}

#[tokio::main]
async fn main() {
    let keypair = identity::Keypair::generate_ed25519();
    let peer_id = keypair.public().into();
    let quic_config = quic::Config::new(&keypair);

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_behaviour(|keypair| Behaviour::new(keypair.to_owned(), peer_id))
        .unwrap()
        .build();

    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

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
        tokio::select! {
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