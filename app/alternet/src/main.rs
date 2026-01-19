#![deny(clippy::unwrap_used)]
#![allow(unused)]

use ::anyhow::Result;
use ::async_trait::async_trait;
use ::orchestrator::network;
use ::orchestrator::runtime;
use ::orchestrator::runtime::{Swarm, SwarmEvent};
use p2p::NetworkBehaviour;
use p2p::swarm::behaviour;

pub struct Alternet<T: NetworkBehaviour> {
    swarm: Swarm<T>,
}

impl<T> From<Swarm<T>> for Alternet<T>
where
    T: NetworkBehaviour,
{
    fn from(swarm: Swarm<T>) -> Self {
        Alternet { swarm }
    }
}

#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct Behaviour {
    alternet: alternet_transport::AlternetBehaviour,
    mdns: libp2p::mdns::tokio::Behaviour,
    // ping: libp2p::ping::Behaviour,
}

#[async_trait]
impl runtime::Node for Alternet<Behaviour> {
    type T = Behaviour;

    fn init_behaviour(
        kp: &p2p::Keypair,
        behaviour: alternet_transport::AlternetBehaviour,
    ) -> <Self as crate::runtime::Node>::T {
        Behaviour {
            alternet: behaviour,
            mdns: libp2p::mdns::Behaviour::new(
                libp2p::mdns::Config::default(),
                kp.public().to_peer_id(),
            )
            .expect("no"),
        }
    }
    fn swarm(&self) -> &Swarm<Self::T> {
        &self.swarm
    }

    fn swarm_mut(&mut self) -> &mut Swarm<Self::T> {
        &mut self.swarm
    }

    async fn run(&mut self, event: SwarmEvent<Self::T>) -> Result<()> {
        let libp2p::swarm::SwarmEvent::Behaviour(ev) = event else {
            return Ok(());
        };
        match ev {
            BehaviourEvent::Alternet(asdf) => todo!(),
            BehaviourEvent::Mdns(mdns) => {
                eprintln!("mdns: {mdns:?}");
            },
        }

        // match event {
        //     event::ConnectionEstablished {
        //         peer_id,
        //         ..
        //     } => {
        //         println!("{} connected to {}", self.swarm.local_peer_id(), peer_id);
        //     },
        //     event::Behaviour(event::Relay(event::RelayCircuitReqAccepted {
        //         src_peer_id,
        //         dst_peer_id
        //     })) => {
        //         println!("{}", src_peer_id);
        //     },
        //     event::Behaviour(event::Dcutr(event::DcutrEvent {
        //         remote_peer_id,
        //         result
        //     })) => {
        //         println!("{}", remote_peer_id);
        //     },
        //     event::Behaviour(event::Mdns(event::MdnsDiscovered(discovered))) => {
        //         for (peer_id, peer_addr) in discovered {
        //             let swarm = self.swarm_mut().behaviour_mut();
        //             swarm.kad.add_address(&peer_id, peer_addr.to_owned());
        //         }
        //     },
        //     event::Behaviour(event::Identify(event::IdentifyReceived {
        //         peer_id,
        //         info,
        //         ..
        //     })) => {
        //         let swarm: &mut p2p::Behaviour = self.swarm_mut().behaviour_mut();
        //         for addr in info.listen_addrs {
        //             swarm.kad.add_address(&peer_id, addr);
        //         }
        //         let _ = swarm.kad.bootstrap();
        //     },
        //     _ => {}
        // }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    use runtime::Node as _;

    let initfn = |swarm: &mut Swarm<Behaviour>| {
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".try_into().unwrap());
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".try_into().unwrap());
        Ok(())
    };

    network::Network::new(vec![])
        .add_runtime(runtime::Runtime::new(Alternet::bootstrap(initfn, None)?))
        .add_runtime(runtime::Runtime::new(Alternet::bootstrap(initfn, None)?))
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
        let node = Alternet::bootstrap(|x| Ok(()), None).expect("");
        let mut runtime = runtime::Runtime::new(node);
        // runtime.poll().await?;
        // needs shutdown mechanism on `Runtime` and `Network`.
        Ok(())
    }
}
