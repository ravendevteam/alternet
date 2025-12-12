use libp2p::futures::StreamExt;
use ::tokio::io;
use ::libp2p::identity;
use ::libp2p::quic;
use ::libp2p::ping;
use ::libp2p::kad;
use ::libp2p::kad::store as kad_store;
use ::libp2p::swarm;

use io::AsyncBufReadExt as _;

::modwire::expose!(
    pub domain
    pub network
);

pub struct Node<T> {
    keypair: identity::Keypair,
    peer_id: ::libp2p::PeerId,
    network: T
}

impl Node<Network> {
    pub async fn bootstrap(self) {
        let quic_config: quic::Config = quic::Config::new(&self.keypair);
        
        let mut swarm: ::libp2p::Swarm<_> = ::libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_quic_config(|_| quic_config)
            .with_behaviour(|_| self.network)
            .unwrap()
            .build();

        let mut stdin: io::Lines<_> = ::tokio::io::BufReader::new(::tokio::io::stdin()).lines();

        loop {
            ::tokio::select! {
                event = swarm.select_next_some() => match event {
                    swarm::SwarmEvent::ConnectionEstablished {
                        peer_id, 
                        connection_id, 
                        endpoint, 
                        num_established, 
                        concurrent_dial_errors, 
                        established_in 
                    } => {

                    },
                    swarm::SwarmEvent::ConnectionClosed {
                        peer_id, 
                        connection_id, 
                        endpoint, 
                        num_established, 
                        cause
                    } => {
                        
                    },
                    _ => {}
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
}

impl Default for Node<Network> {
    fn default() -> Self {
        let keypair: identity::Keypair = identity::Keypair::generate_ed25519();
        let peer_id: ::libp2p::PeerId = keypair.public().into();
        let network: Network = Network::new(peer_id);
        Self {
            keypair,
            peer_id,
            network
        }
    }
}

// node -> network -> impls and logic -> bootstrap -> use methods on node and network on events
// possibly add express api to add components that require swarm and other stuff