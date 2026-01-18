use super::*;

#[async_trait]
pub trait Node
where
    Self: Sized {
    fn new(swarm: impl Into<Swarm>) -> Self;

    fn bootstrap() -> Result<Self> {
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
        let ret: Self = Self::new(swarm);
        Ok(ret)
    }

    fn swarm(&self) -> &Swarm;
    fn swarm_mut(&mut self) -> &mut Swarm;

    async fn run(&mut self, event: SwarmEvent) -> Result<()>;
}

pub struct Runtime<T> {
    node: T,
    running: bool
}

impl<T> Runtime<T> {
    pub fn new(node: impl Into<T>) -> Self {
        let node: T = node.into();
        Self {
            node,
            running: true
        }
    }
}

impl<T> Runtime<T>
where
    T: Node {
    pub fn running(&self) -> bool {
        self.running
    }

    pub fn shutdown(&mut self) {
        println!("shutting down");
        self.running = false;
    }

    pub async fn poll(&mut self) -> Result<()> {
        while self.running() {
            let event: SwarmEvent = self.node.swarm_mut().select_next_some().await;
            self.node.run(event).await?; 
        }
        Ok(())
    }
}

#[async_trait]
impl<T> network::Runtime for Runtime<T> 
where
    T: Node,
    T: Send {
    async fn launch(mut self: Box<Self>) -> Result<()> {
        self.poll().await
    }
}