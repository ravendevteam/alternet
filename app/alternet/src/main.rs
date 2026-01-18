#![deny(clippy::unwrap_used)]
#![allow(unused)]

use p2p::*;
use orchestrator::network;
use orchestrator::runtime;
use async_trait::async_trait;
use anyhow::Result;

pub struct Alternet {
    swarm: Swarm
}

#[async_trait]
impl runtime::Node for Alternet {
    fn new(swarm: impl Into<Swarm>) -> Self {
        let swarm: Swarm = swarm.into();
        Self {
            swarm
        }
    }

    fn swarm(&self) -> &Swarm {
        &self.swarm
    }

    fn swarm_mut(&mut self) -> &mut Swarm {
        &mut self.swarm
    }

    async fn run(&mut self, event: SwarmEvent) -> Result<()> {
        match event {
            event::ConnectionEstablished {
                peer_id,
                ..
            } => {
                println!("{} connected to {}", self.swarm.local_peer_id(), peer_id);
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

    network::Network::new(vec![])
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