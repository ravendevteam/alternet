use ::anyhow::Result;
use ::async_trait::async_trait;
use ::futures::StreamExt as _;
use ::libp2p::core::muxing::StreamMuxerBox;
use ::libp2p::identity::Keypair;
use ::libp2p::{Transport, noise, quic, tcp, yamux};
use libp2p::swarm::NetworkBehaviour;

pub type Swarm<T> = libp2p::Swarm<T>;
pub type SwarmEvent<T> = libp2p::swarm::SwarmEvent<<T as NetworkBehaviour>::ToSwarm>;

fn get_alternet_stuff(
    keypair: &Keypair,
) -> (
    libp2p::core::transport::Boxed<(libp2p::PeerId, libp2p::core::muxing::StreamMuxerBox)>,
    alternet_transport::AlternetBehaviour,
) {
    let quic_config: quic::Config = quic::Config::new(&keypair);
    let tcp_config: tcp::Config = tcp::Config::default();

    let tcp_transport = libp2p::tcp::tokio::Transport::new(tcp_config)
        .upgrade(libp2p::core::upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(&keypair).expect("this really should work"))
        .multiplex(yamux::Config::default())
        .map(|(p, c), _| (p, StreamMuxerBox::new(c)));

    let quic_transport = libp2p::quic::tokio::Transport::new(quic_config)
        .map(|(peer_id, muxer), _| (peer_id, libp2p::core::muxing::StreamMuxerBox::new(muxer)));

    let dns_transporter = libp2p::dns::tokio::Transport::system;

    let (alternet_transporter, alternet_behaviour) = alternet_transport::new(
        &keypair,
        dns_transporter(
            tcp_transport
                .or_transport(quic_transport)
                .map(|either, _| either.into_inner()),
        )
        .expect("dns should work"),
        alternet_transport::AlternetOptions { relay: false },
    );

    let transport_timeout = libp2p::core::transport::timeout::TransportTimeout::new(
        alternet_transporter.map(|either, _| either),
        std::time::Duration::from_secs(10),
    );

    return (transport_timeout.boxed(), alternet_behaviour);
}

#[async_trait]
pub trait Node
where
    Self: Sized + From<Swarm<Self::T>>,
{
    type T: NetworkBehaviour + Send;

    fn init_behaviour(
        keypair: &Keypair,
        swarm: alternet_transport::AlternetBehaviour,
    ) -> <Self as Node>::T;

    fn bootstrap<F>(init: F, keypair: Option<Keypair>) -> Result<Self>
    where
        F: FnOnce(&mut Swarm<Self::T>) -> Result<()>,
    {
        let keypair: Keypair = keypair.unwrap_or_else(Keypair::generate_ed25519);

        let (transport, behaviour) = get_alternet_stuff(&keypair);

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_other_transport(|_| transport)
            .expect("fuck")
            .with_behaviour(|kp| Self::init_behaviour(kp, behaviour))
            .expect("fuck")
            .build();

        init(&mut swarm)?;

        Ok(swarm.into())
    }

    fn swarm(&self) -> &Swarm<Self::T>;
    fn swarm_mut(&mut self) -> &mut Swarm<Self::T>;

    async fn run(&mut self, event: SwarmEvent<Self::T>) -> Result<()>;
}

pub struct Runtime<T> {
    node: T,
    running: bool,
}

impl<T> Runtime<T> {
    pub fn new(node: T) -> Self {
        let node: T = node.into();
        Self {
            node,
            running: true,
        }
    }
}

impl<T> Runtime<T>
where
    T: Node,
{
    pub fn running(&self) -> bool {
        self.running
    }

    pub fn shutdown(&mut self) {
        println!("shutting down");
        self.running = false;
    }

    pub async fn poll(&mut self) -> Result<()> {
        while self.running() {
            let event: SwarmEvent<<T as Node>::T> = self.node.swarm_mut().select_next_some().await;
            self.node.run(event).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<T> crate::network::Runtime for Runtime<T>
where
    T: Node + Send,
{
    async fn launch(mut self: Box<Self>) -> Result<()> {
        self.poll().await
    }
}
