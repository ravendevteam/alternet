#[cfg(not(any(feature = "client", feature = "server", feature = "relay", feature = "bootstrap")))]
compile_error!("Enable exactly one of `client`, `server`, `relay`, or `bootstrap` features.");

#[cfg(all(feature = "client", any(feature = "server", feature = "relay", feature = "bootstrap")))]
compile_error!("Only one of `client`, `server`, `relay`, or `bootstrap` may be enabled at a time.");

#[cfg(all(feature = "server", any(feature = "client", feature = "relay", feature = "bootstrap")))]
compile_error!("Only one of `client`, `server`, `relay`, or `bootstrap` may be enabled at a time.");

#[cfg(all(feature = "relay", any(feature = "client", feature = "server", feature = "bootstrap")))]
compile_error!("Only one of `client`, `server`, `relay`, or `bootstrap` may be enabled at a time.");

use clap::Parser;
use tokio::io::AsyncBufReadExt as _;
use libp2p::swarm;
use libp2p::identify;
use libp2p::identity;
use libp2p::kad;
use libp2p::gossipsub;
use libp2p::request_response;
use libp2p::quic;
use libp2p::noise;
use libp2p::yamux;
use libp2p::futures::StreamExt as _;
use libp2p::relay;

#[cfg(any(feature = "client", feature = "server"))]
use libp2p::dcutr;

mod config;
mod env_key;
mod grpc;
mod sub_system;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type Swarm = swarm::Swarm<Behaviour>;
type SwarmEvent = swarm::SwarmEvent<BehaviourEvent>;

enum Grpc {
    
}

#[derive(Debug)]
#[derive(clap::Parser)]
#[command(author)]
#[command(version)]
#[command(about)]
struct Cli {
    #[arg(long)]
    pub grpc_endpoint: Option<std::net::SocketAddr>,
    #[arg(long)]
    pub dial: Option<Vec<libp2p::Multiaddr>>
}

// MARK: Behaviour

#[derive(swarm::NetworkBehaviour)]
struct Behaviour {
    #[cfg(feature = "relay")]
    pub relay: relay::Behaviour,
    #[cfg(any(feature = "client", feature = "server"))]
    pub relay_client: relay::client::Behaviour,
    #[cfg(any(feature = "client", feature = "server"))]
    pub dcutr: dcutr::Behaviour,
    #[cfg(any(feature = "client", feature = "server", feature = "relay", feature = "bootstrap"))]
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
    #[cfg(any(feature = "client", feature = "server", feature = "relay", feature = "bootstrap"))]
    pub identify: identify::Behaviour
}

// MARK: Main

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    fern::Dispatch::new()
        .format(|out, message, record| {
            use colored::Colorize as _;

            let record_time: std::time::SystemTime = std::time::SystemTime::now();
            let record_time: humantime::Rfc3339Timestamp = humantime::format_rfc3339(record_time);
            let record_level: colored::ColoredString = match record.level() {
                log::Level::Debug => record.level().to_string().blue().bold(),
                log::Level::Trace => record.level().to_string().magenta().bold(),
                log::Level::Error => record.level().to_string().red().bold(),
                log::Level::Info => record.level().to_string().green().bold(),
                log::Level::Warn => record.level().to_string().yellow().bold()
            };
            let record_target: &str = record.target();
            let s: std::fmt::Arguments<'_> = format_args!("[{} {} {}] {}", record_time, record_level, record_target, message);
            out.finish(s);
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;

    log::info!("Booting");

    let conf: Option<_> = config::Config::from_toml()?;

    let dial: Vec<_> = if let Some(dial) = cli.dial {
        dial
    } else if let Some(conf) = &conf && let Some(dial) = &conf.dial {
        dial.to_owned()
    } else {
        vec![]
    };

    let version: &str = env!("CARGO_PKG_VERSION");
    let protocol_version: String = format!("/an/{}", version);

    #[cfg(feature = "client")]
    let agent_version: String = format!("an-client/{}", version);

    #[cfg(feature = "server")]
    let agent_version: String = format!("an-server/{}", version);

    #[cfg(feature = "relay")]
    let agent_version: String = format!("an-relay/{}", version);

    #[cfg(feature = "bootstrap")]
    let agent_version: String = format!("an-bootstrap/{}", version);

    #[cfg(feature = "client")]
    let identify_cache_size: usize = if let Some(conf) = &conf
    && let Some(client) = &conf.client
    && let Some(identity_cache_size) = client.identity_cache_size {
        identity_cache_size
    } else {
        1000
    };

    #[cfg(feature = "server")]
    let identify_cache_size: usize = if let Some(conf) = conf 
    && let Some(server) = conf.server 
    && let Some(identity_cache_size) = server.identity_cache_size {
        identity_cache_size
    } else {
        2000
    };

    #[cfg(feature = "relay")]
    let identify_cache_size: usize = if let Some(config) = config
    && let Some(relay) = config.relay
    && let Some(identity_cache_size) = relay.identity_cache_size {
        identity_cache_size
    } else {
        5000
    };

    #[cfg(feature = "bootstrap")]
    let identify_cache_size: usize = 50000;

    #[cfg(feature = "client")]
    let identify_interval: std::time::Duration = std::time::Duration::from_mins(20);

    #[cfg(feature = "server")]
    let identify_interval: std::time::Duration = std::time::Duration::from_mins(10);

    #[cfg(feature = "relay")]
    let identify_interval: std::time::Duration = std::time::Duration::from_mins(10);

    #[cfg(feature = "bootstrap")]
    let identify_interval: std::time::Duration = std::time::Duration::from_mins(1);

    let local_keypair: identity::Keypair = identity::Keypair::generate_ed25519();
    let local_public_key: identity::PublicKey = local_keypair.public();
    let local_peer_id: libp2p::PeerId = local_keypair.public().into();

    log::info!("Peer identity initialized: {:?}", local_peer_id);

    let mut quic_config: quic::Config = quic::Config::new(&local_keypair);
    quic_config.handshake_timeout = std::time::Duration::from_millis(3000);
    quic_config.keep_alive_interval = std::time::Duration::from_millis(3000);
    quic_config.max_concurrent_stream_limit = 3000;
    quic_config.max_connection_data = 3000;
    quic_config.max_idle_timeout = 3000;
    quic_config.max_stream_data = 300;

    #[cfg(any(feature = "client", feature = "server"))] 
    let mut swarm: libp2p::Swarm<_> = libp2p::SwarmBuilder::with_existing_identity(local_keypair)
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|_, relay_client| {
            let kad_store: kad::store::MemoryStore = kad::store::MemoryStore::new(local_peer_id);
            let mut kad: kad::Behaviour<_> = kad::Behaviour::new(local_peer_id, kad_store);
            
            #[cfg(feature = "client")]
            kad.set_mode(Some(kad::Mode::Client));

            #[cfg(feature = "server")]
            kad.set_mode(Some(kad::Mode::Server));

            let dcutr: dcutr::Behaviour = dcutr::Behaviour::new(local_peer_id);
        
            #[cfg(feature = "client")]
            let identify_config: identify::Config = identify::Config::new(protocol_version, local_public_key)
                .with_agent_version(agent_version)
                .with_cache_size(identify_cache_size)
                .with_hide_listen_addrs(false)
                .with_interval(identify_interval)
                .with_push_listen_addr_updates(true);

            #[cfg(feature = "server")]
            let identify_config: identify::Config = identify::Config::new(protocol_version, local_public_key)
                .with_agent_version(agent_version)
                .with_cache_size(identify_cache_size)
                .with_hide_listen_addrs(false)
                .with_interval(identify_interval)
                .with_push_listen_addr_updates(true);

            let identify: identify::Behaviour = identify::Behaviour::new(identify_config);

            Behaviour {
                relay_client,
                dcutr,
                kad,
                identify
            }
        })?
        .build();
    
    #[cfg(feature = "relay")]
    let mut swarm: libp2p::Swarm<_> = libp2p::SwarmBuilder::with_existing_identity(local_keypair)
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_behaviour(|_| {
            let relay_config: relay::Config = relay::Config {
                max_circuit_bytes: 1000,
                max_circuit_duration: std::time::Duration::from_millis(3000),
                max_reservations: 10,
                max_circuits: 0,
                max_circuits_per_peer: 0,
                max_reservations_per_peer: 0,
                reservation_duration: std::time::Duration::from_millis(3000),
                reservation_rate_limiters: vec![],
                circuit_src_rate_limiters: vec![]
            };
            let relay: relay::Behaviour = relay::Behaviour::new(local_peer_id, relay_config);

            let kad_store: kad::store::MemoryStore = kad::store::MemoryStore::new(local_peer_id);
            let mut kad: kad::Behaviour<_> = kad::Behaviour::new(local_peer_id, kad_store);
            
            kad.set_mode(Some(kad::Mode::Server));

            let identify_config: identify::Config = identify::Config::new(protocol_version, local_public_key)
                .with_agent_version(agent_version)
                .with_cache_size(identify_cache_size)
                .with_hide_listen_addrs(false)
                .with_interval(identify_interval)
                .with_push_listen_addr_updates(true);
            let identify: identify::Behaviour = identify::Behaviour::new(identify_config);

            Behaviour {
                relay,
                kad,
                identify
            }
        })
        .expect("")
        .build();

    #[cfg(feature = "bootstrap")]
    let mut swarm: libp2p::Swarm<_> = libp2p::SwarmBuilder::with_existing_identity(local_keypair)
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_behaviour(|_| {
            let kad_store: kad::store::MemoryStore = kad::store::MemoryStore::new(local_peer_id);
            let mut kad: kad::Behaviour<_> = kad::Behaviour::new(local_peer_id, kad_store);
            
            kad.set_mode(Some(kad::Mode::Server));

            let identify_config: identify::Config = identify::Config::new(protocol_version, local_public_key)
                .with_agent_version(agent_version)
                .with_cache_size(identify_cache_size)
                .with_hide_listen_addrs(false)
                .with_interval(identify_interval)
                .with_push_listen_addr_updates(true);
            let identify: identify::Behaviour = identify::Behaviour::new(identify_config);            

            Behaviour {
                kad,
                identify
            }
        })?
        .build();

    swarm.listen_on("/ip4/0.0.0.0/udp/4001/quic-v1".parse()?)?;

    let (mut sx, mut rx) = tokio::sync::mpsc::channel::<Grpc>(1000);

    let grpc_endpoint: std::net::SocketAddr = if let Some(grpc_endpoint) = cli.grpc_endpoint {
        grpc_endpoint
    } else if let Some(conf) = &conf && let Some(grpc) = &conf.grpc_endpoint {
        grpc.to_owned()
    } else {
        "0.0.0.0:8080".parse()?
    };

    let grpc_server: grpc::Node = grpc::Node {};
    let grpc_server: grpc::proto::node_server::NodeServer<_> = grpc::proto::node_server::NodeServer::new(grpc_server);
    let grpc = tonic::transport::Server::builder()
        .add_service(grpc_server)
        .serve(grpc_endpoint);

    let ctrl_c = tokio::signal::ctrl_c();

    tokio::pin!(grpc);
    tokio::pin!(ctrl_c);
    
    log::info!("finished booting");

    let bootstrap: sub_system::bootstrap::Bootstrap = sub_system::bootstrap::Bootstrap::builder()
        .timeout_duration(std::time::Duration::from_secs(8))
        .min_peers(2)
        .bootstrap_addrs(dial)
        .build();

    let connection_manager: sub_system::connection_manager::ConnectionManager = sub_system::connection_manager::ConnectionManager::builder()
        .target_peer_count(2)
        .min_retry_delay(std::time::Duration::from_secs(8))
        .max_retry_delay(std::time::Duration::from_secs(32))
        .build();

    let routing_monitor: sub_system::routing_monitor::RoutingMonitor = sub_system::routing_monitor::RoutingMonitor::builder()
        .sample_interval(std::time::Duration::from_secs(30))
        .collapse_threshold(2)
        .churn_window(std::time::Duration::from_secs(30))
        .build();

    let mut sub_system_bus: sub_system::Bus = sub_system::Bus::default();
    sub_system_bus.add_system(bootstrap);
    sub_system_bus.add_system(connection_manager);
    sub_system_bus.add_system(routing_monitor);

    loop {
        tokio::select!(
            _ = &mut ctrl_c => break,
            _ = &mut grpc => break,
            event = swarm.select_next_some() => sub_system_bus.receive(&mut swarm, sub_system::Event::new(event)),
            Some(opcode) = rx.recv() => sub_system_bus.receive(&mut swarm, sub_system::Event::new(opcode))
        );
    }

    Ok(())
}