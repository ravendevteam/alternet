#![deny(clippy::unwrap_used)]
#![allow(unused)]

use std::marker::PhantomData;
use p2p::*;
use orchestrator::network;
use orchestrator::runtime;
use async_trait::async_trait;
use anyhow::Result;

mod ipv4;
mod record;
mod records;

#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Alternet(Swarm);

impl Alternet {
    pub fn kad(&self) -> &kad::Behaviour<kad::store::MemoryStore> {
        &self.behaviour().kad
    }

    pub fn add_record(&mut self, record: impl Into<record::Record<()>>) {
        
        let record: record::Record<()> = record.into();
        let record: kad::Record = kad::Record {
            key:,
            value: vec![],
            publisher: None,
            expires: None
        };
        let swarm_mut: &mut Behaviour = self.behaviour_mut();
        swarm_mut.kad.put_record( , kad::Quorum::Majority);
    }
}

#[async_trait]
impl runtime::Node for Alternet {
    fn new(swarm: impl Into<Swarm>) -> Self {
        let swarm: Swarm = swarm.into();
        Self(swarm)
    }

    fn swarm(&self) -> &Swarm {
        self
    }

    fn swarm_mut(&mut self) -> &mut Swarm {
        self
    }

    async fn run(&mut self, event: SwarmEvent) -> Result<()> {
        match event {
            event::ConnectionEstablished {
                peer_id,
                ..
            } => {
                println!("{} connected to {}", self.local_peer_id(), peer_id);
            },
            event::Behaviour(event::Relay(event::RelayCircuitReqAccepted {
                src_peer_id,
                dst_peer_id 
            })) => {
                println!("{}", src_peer_id);
            },
            event::Behaviour(event::Dcutr(event::DcutrEvent {
                remote_peer_id,
                result
            })) => {
                println!("{}", remote_peer_id);
            },
            event::Behaviour(event::Mdns(event::MdnsDiscovered(discovered))) => {
                for (peer_id, peer_addr) in discovered {
                    let swarm = self.swarm_mut().behaviour_mut();
                    swarm.kad.add_address(&peer_id, peer_addr.to_owned());
                }
            },
            event::Behaviour(event::Identify(event::IdentifyReceived {
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
            _ => {}
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    use runtime::Node as _;

    network::Network::default()
        .add_runtime(runtime::Runtime::<Alternet>::new(Alternet::bootstrap()?))
        .add_runtime(runtime::Runtime::<Alternet>::new(Alternet::bootstrap()?))
        .connect()
        .await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn should_spawn_a_node() -> Result<()> {
        use runtime::Node as _;
        let node: Alternet = Alternet::bootstrap().expect("");
        let mut runtime: runtime::Runtime<Alternet> = runtime::Runtime::new(node);
        // runtime.poll().await?;
        // needs shutdown mechanism on `Runtime` and `Network`.
        Ok(())
    }
}