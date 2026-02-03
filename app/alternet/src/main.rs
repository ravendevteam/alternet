#![deny(clippy::unwrap_used)]
#![allow(unused)]

use core::time;
use std::marker::PhantomData;
use p2p::*;
use async_trait::async_trait;
use anyhow::Result;

use tokio::sync::oneshot as tokio_oneshot;

mod key;
mod opcode;
mod record;


// if two people publish the same domain and point to different places...??
// 
// first come, first serve??
//
// first is not globally observable...
//
// network partitions
// clock skew
// simultaneous publishes
// eclipse attacks
// closest peers disagreement
//

// cryptographic ownership - The owner of a domain is whoever controls the private key bound to that name.
// ens, ipns, handshake


// domains are case sensitive?? is kebab case allowed?



// 1. use gossipsub to broadcast data - and republish counts??

// *** this
// 2. use peer_id so people could have multiple domains
//    - would also solve domain economic problems
//    - would not really need consensus because search engine would handle this

#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Main {
    #[deref]
    #[deref_mut]
    swarm: Swarm,
    put_pending: std::collections::HashMap<kad::QueryId, tokio_oneshot::Sender<()>>,
    get_pending: std::collections::HashMap<kad::QueryId, tokio_oneshot::Sender<Result<Option<kad::Record>>>>
}

impl Main {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let keypair: Keypair = Keypair::generate_ed25519();
        let peer_id: PeerId = keypair.public().into();
        let quic_config: quic::Config = quic::Config::new(&keypair);
        let tcp_config: tcp::Config = tcp::Config::default();
        let mut swarm: Swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.to_owned())
            .with_tokio()
            .with_tcp(tcp_config, noise::Config::new, yamux::Config::default)?
            .with_quic_config(|_| quic_config)
            .with_behaviour(move |_| {
                let local_peer_id: PublicKey = keypair.public();
                let local_peer_id: PeerId = PeerId::from(local_peer_id);
                let identify_config: identify::Config = identify::Config::new("/an/1.0.0".to_owned(), keypair.public());
                let identify: identify::Behaviour = identify::Behaviour::new(identify_config);
                let ping_config: ping::Config = ping::Config::new();
                let ping: ping::Behaviour = ping::Behaviour::new(ping_config);
                let kad_store: kad::store::MemoryStore = kad::store::MemoryStore::new(peer_id);
                let kad: kad::Behaviour<kad::store::MemoryStore> = kad::Behaviour::new(peer_id, kad_store);
                let gossipsub_key: gossipsub::MessageAuthenticity = gossipsub::MessageAuthenticity::Signed(keypair.to_owned());
                let gossipsub_config: gossipsub::Config = gossipsub::Config::default();
                let gossipsub: gossipsub::Behaviour = gossipsub::Behaviour::new(gossipsub_key, gossipsub_config).expect("key and config should be correct whilst building the gossipsub behaviour");
                let relay_config: relay::Config = relay::Config::default();
                let relay: relay::Behaviour = relay::Behaviour::new(local_peer_id, relay_config);
                let dcutr: dcutr::Behaviour = dcutr::Behaviour::new(local_peer_id);
                let mdns_config: mdns::Config = mdns::Config::default();
                let mdns: mdns::tokio::Behaviour = mdns::tokio::Behaviour::new(mdns_config, local_peer_id).expect("");
                Behaviour {
                    identify,
                    ping,
                    kad,
                    gossipsub,
                    relay,
                    dcutr,
                    mdns
                }
            })?
            .build();
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().expect("multi address should be correct")).expect("swarm should be able to listen on given multi address");
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        let put_pending: std::collections::HashMap<_, _> = std::collections::HashMap::default();
        let get_pending: std::collections::HashMap<_, _> = std::collections::HashMap::default();
        let ret: Self = Self {
            swarm,
            put_pending,
            get_pending
        };
        Ok(ret)
    }
}

// we can get rid of these and relocate them directly into the opcode processing system.
impl Main {
    pub fn resolve(&mut self, domain: String) -> tokio_oneshot::Receiver<Result<Option<kad::Record>>> {
        let key: kad::RecordKey = domain.into_bytes().into();
        let swarm: &mut Behaviour = self.behaviour_mut();
        let query_id: kad::QueryId = swarm.kad.get_record(key);
        let (sx, rx) = tokio_oneshot::channel();
        self.get_pending.insert(query_id, sx);
        rx
    }
}

#[async_trait]
impl orchestrator::Node for Main {
    type Opcode = opcode::Opcode;

    fn swarm(&self) -> &Swarm {
        self
    }

    fn swarm_mut(&mut self) -> &mut Swarm {
        self
    }

    async fn receive(&mut self, event: SwarmEvent) -> Result<(), Box<dyn std::error::Error>> {
        use event::*;
        match event {
            event::ConnectionEstablished {
                peer_id,
                ..
            } => {
                println!("{} connected to {}", self.local_peer_id(), peer_id);
            }

            event::Behaviour(event::Relay(event::RelayCircuitReqAccepted {
                src_peer_id,
                dst_peer_id 
            })) => {
                println!("{}", src_peer_id);
            }

            event::Behaviour(event::Dcutr(event::DcutrEvent {
                remote_peer_id,
                result
            })) => {
                println!("{}", remote_peer_id);
            }

            event::Behaviour(event::Mdns(event::MdnsDiscovered(discovered))) => {
                for (peer_id, peer_addr) in discovered {
                    let swarm = self.swarm_mut().behaviour_mut();
                    swarm.kad.add_address(&peer_id, peer_addr.to_owned());
                }
            },
            Behaviour(Identify(IdentifyReceived {
                peer_id,
                info,
                ..
            })) => {
                let swarm: &mut p2p::Behaviour = self.swarm_mut().behaviour_mut();
                for addr in info.listen_addrs {
                    swarm.kad.add_address(&peer_id, addr);
                }
                let _ = swarm.kad.bootstrap();
            },
            Behaviour(Kad(KadOutboundQueryProgressed {
                id,
                result,
                stats,
                step
            })) => {
                match result {
                    kad::QueryResult::PutRecord(Ok(_)) if step.last => {
                        if let Some(sx) = self.put_pending.remove(&id) {
                            let _ = sx.send(());
                        }
                    },
                    _ => {}
                }
                if let Some(sx) = self.get_pending.remove(&id) {
                    match result {
                        kad::QueryResult::PutRecord(result) if step.last => {
                            if let Some(sx) = self.put_pending.remove(&id) {
                                let _ = sx.send(());
                            }
                        },
                        kad::QueryResult::GetRecord(Ok(
                            kad::GetRecordOk::FoundRecord(peer_record)
                        )) => {
                            let _ = sx.send(Ok(Some(peer_record.record)));
                        }

                        kad::QueryResult::GetRecord(Ok(
                            kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. }
                        )) => {
                            // FIX: Use 'sx' directly
                            let _ = sx.send(Ok(None));
                        }

                        kad::QueryResult::GetRecord(Err(e)) => {
                            // FIX: Use 'sx' directly
                            let _ = sx.send(Err(e.into()));
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }

    // since the node is owned by the runtime, we need to use opcodes to communicate with it asynchronously.
    #[allow(clippy::unwrap_used)]
    async fn receive_opcode(&mut self, opcode: Self::Opcode) -> Result<(), Box<dyn std::error::Error>> {
        match opcode {
            // anyone can just register how to resolve conflict??
            // we need a consensus system.
            opcode::Opcode::Put(opcode) => {
                let publisher: PeerId = self.local_peer_id().to_owned();
                let duration: std::time::Duration = std::time::Duration::from_hours(24);
                let expiration: std::time::Instant = std::time::Instant::now().checked_add(duration).ok_or("overflow")?;
                let record: record::Record = record::Record::new(opcode.domain.to_owned(), publisher, expiration);
                let record: kad::Record = record.into();
                let swarm: &mut Behaviour = self.behaviour_mut();
                let query_id: kad::QueryId = swarm.kad.put_record(record, kad::Quorum::One)?;
                let (sx, rx) = tokio_oneshot::channel();
                self.put_pending.insert(query_id, sx);
                tokio::spawn(async move {
                    if rx.await.is_ok() {
                        println!("DomainRegistration {}", opcode.domain);
                    }
                });
                Ok(())
            },
            opcode::Opcode::Resolve(opcode) => {
                let rx = self.resolve(opcode.domain);
                tokio::spawn(async move {
                    if let Ok(Ok(Some(record))) = rx.await {
                        println!("DomainResolved {:?}", record.value);
                    }
                });
                Ok(())
            },
            _ => Ok(())
        }
    }
}

#[allow(clippy::unwrap_used)]
#[tokio::main]
async fn main() -> Result<()> {
    let count: usize = 8;
    let mut network: Vec<_> = Vec::new();
    for _ in 0..=count {
        network.push(Main::new().unwrap());
    }
    let network: Vec<_> = network
        .into_iter()
        .map(|node| {
            orchestrator::Runtime::spawn(node)
        })
        .collect();
    println!("Waiting for bootstrap");
    let duration: std::time::Duration = std::time::Duration::from_millis(3000);
    tokio::time::sleep(duration).await;
    println!("Done waiting");
    let opcode: opcode::Put = opcode::Put {
        domain: "nordvpn".to_owned()
    };
    let opcode: opcode::Opcode = opcode::Opcode::Put(opcode);
    network.first().unwrap().send(opcode);
    let duration: std::time::Duration = std::time::Duration::from_millis(3000);
    tokio::time::sleep(duration).await;
    // domains are case sensitive, needs to be fixed
    let opcode: opcode::Resolve = opcode::Resolve {
        domain: "nordvpn".to_owned()
    };
    let opcode: opcode::Opcode = opcode::Opcode::Resolve(opcode);
    network.first().unwrap().send(opcode);
    let suspension: orchestrator::suspend::Suspend<_> = network.into();
    suspension.wait_for(std::time::Duration::from_secs(10)).await;
    Ok(())
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod test {
    use super::*;
    use orchestrator::Node as _;

    // local connections
    #[tokio::test]
    async fn should_spawn_a_node() -> Result<()> {
        let count: usize = 8;
        let mut network: Vec<_> = Vec::new();
        for _ in 0..=count {
            network.push(Main::new().unwrap());
        }
        let network: Vec<_> = network
            .into_iter()
            .map(|node| {
                orchestrator::Runtime::spawn(node)
            })
            .collect();
        let opcode: opcode::Put = opcode::Put {
            domain: "nordvpn".to_owned()
        };
        let opcode: opcode::Opcode = opcode::Opcode::Put(opcode);
        network.first().unwrap().send(opcode);
        let suspension: orchestrator::suspend::Suspend<_> = network.into();
        suspension.wait_for(std::time::Duration::from_secs(10)).await;
        Ok(())
    }

    // relays
    // dht tests

    #[tokio::test]
    async fn should_store_kad_record() -> Result<(), Box<dyn std::error::Error>> {

        Ok(())
    }
}