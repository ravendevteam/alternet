#![deny(clippy::unwrap_used)]
#![allow(unused)]

use std::sync::Arc;

use ::anyhow::Result;
use ::async_trait::async_trait;
use ::orchestrator::network;
use ::orchestrator::runtime;
use ::orchestrator::runtime::{Swarm, SwarmEvent};
use p2p::Multiaddr;
use p2p::NetworkBehaviour;
use p2p::StreamProtocol;
use p2p::future::AsyncRead;
use p2p::future::AsyncWrite;
use p2p::future::StreamExt;
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
}

pub struct Config {
    mdns: bool,
}
fn alternet<B: NetworkBehaviour>(
    cfg: Config,
    from_alternet: impl FnOnce(alternet_transport::AlternetBehaviour) -> B,
) -> libp2p::Swarm<B> {
    todo!()
}

async fn asdf_main() {
    #[derive(libp2p::swarm::NetworkBehaviour)]
    struct MyBehavior {
        alternet: alternet_transport::AlternetBehaviour,
        http: libp2p_stream::Behaviour,
    }

    let swarm = alternet(Config { mdns: true }, |alternet| MyBehavior {
        alternet,
        http: libp2p_stream::Behaviour::new(),
    });

    let mut x = swarm
        .behaviour()
        .http
        .new_control()
        .accept(StreamProtocol::new("http"))
        .expect("not already registered");

    let mut server = tide::Server::new();
    server
        .at("/")
        .serve_dir("/home/rob9315/Desktop/website/public/")
        .unwrap();
    let server = std::sync::Arc::new(server);

    while let Some((peer, stream)) = x.next().await {
        let stream = async_dup::Arc::new(async_dup::Mutex::new(stream));
        let _join_handle = tokio::spawn(my_handler(server.clone(), stream)).await;
    }
}
async fn server_main() {
    #[derive(libp2p::swarm::NetworkBehaviour)]
    struct MyBehavior {
        alternet: alternet_transport::AlternetBehaviour,
        http_client: libp2p_stream::Behaviour,
    }

    let mut swarm = alternet(Config { mdns: true }, |alternet| MyBehavior {
        alternet,
        http_client: libp2p_stream::Behaviour::new(),
    });

    swarm
        .dial(Multiaddr::try_from("/dns/coolblog.pets").unwrap())
        .expect("asdf");

    // swarm.behaviour_mut().alternet.

    let mut x = swarm
        .behaviour()
        .http_client
        .new_control()
        .accept(StreamProtocol::new("http"))
        .expect("not already registered");


    let mut server = tide::Server::new();
    server
        .at("/")
        .serve_dir("/home/rob9315/Desktop/website/public/")
        .unwrap();
    let server = std::sync::Arc::new(server);

    loop {
        tokio::select! {
            _ = swarm.select_next_some() => {},
            conn = x.next() => {
                if let Some((peer, stream)) = conn {
                    let stream = async_dup::Arc::new(async_dup::Mutex::new(stream));
                    let _join_handle = tokio::spawn(my_handler(server.clone(), stream)).await;
                }
            }
        }
    }

}

async fn my_handler<S>(
    server: Arc<tide::Server<()>>,
    stream: S,
) -> std::result::Result<(), tide::Error>
where
    S: AsyncRead + AsyncWrite + Send + Unpin + Sync + Clone + 'static,
{
    Ok(async_h1::accept(stream, |req| server.respond(req)).await?)
}



//                      an://coolblog.pets/index.html
//                      /an/coolblog.pets/http/index.html
//                      /p2p-circuit/p2p/Qmyourthing/http/index.html

#[async_trait]
impl runtime::Node for Alternet<Behaviour> {
    type T = Behaviour;

    fn init_behaviour(
        kp: &p2p::Keypair,
        behaviour: alternet_transport::AlternetBehaviour,
    ) -> <Self as crate::runtime::Node>::T {
        Behaviour {
            alternet: behaviour,
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
            BehaviourEvent::Alternet(asdf) => {}
            // BehaviourEvent::Mdns(mdns) => {
            //     eprintln!("mdns: {mdns:?}");
            // }
        }

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
