cfg_if::cfg_if!(
    if #[cfg(not(any(
        feature = "bootstrap",
        feature = "client",
        feature = "server",
        feature = "relay",
        feature = "malicious_bootstrap",
        feature = "malicious_client",
        feature = "malicious_server",
        feature = "malicious_relay"
    )))] {
        compile_error!("Enable exactly one of `client`, `server`, `relay`, `bootstrap`, or their malicious variants.");
    } else if #[cfg(all(feature = "bootstrap", any(
        feature = "client",
        feature = "server",
        feature = "relay",
        feature = "malicious_bootstrap",
        feature = "malicious_client",
        feature = "malicious_server",
        feature = "malicious_relay"
    )))] {
        compile_error!("Only one of `client`, `server`, `relay`, `bootstrap`, or their malicious variants may be enabled at a time.");
    } else if #[cfg(feature = "client", any(
        feature = "bootstrap",
        feature = "server",
        feature = "relay",
        feature = "malicious_bootstrap",
        feature = "malicious_client",
        feature = "malicious_server",
        feature = "malicious_relay"
    ))] {
        compile_error!("Only one of `client`, `server`, `relay`, `bootstrap`, or their malicious variants may be enabled at a time.");
    } else if #[cfg(feature = "server", any(
        feature = "bootstrap",
        feature = "server",
        feature = "relay",
        feature = "malicious_bootstrap",
        feature = "malicious_client",
        feature = "malicious_server",
        feature = "malicious_relay"
    ))] {
        compile_error!("Only one of `client`, `server`, `relay`, `bootstrap`, or their malicious variants may be enabled at a time.");
    } else if #[cfg(feature = "relay", any(
        feature = "client",
        feature = "server",
        feature = "bootstrap",
        feature = "malicious_bootstrap",
        feature = "malicious_client",
        feature = "malicious_server",
        feature = "malicious_relay"
    ))] {
        compile_error!("Only one of `client`, `server`, `relay`, `bootstrap`, or their malicious variants may be enabled at a time.");
    } else if #[cfg(feature = "malicious_bootstrap", any(
        feature = "client",
        feature = "server",
        feature = "bootstrap",
        feature = "relay",
        feature = "malicious_client",
        feature = "malicious_server",
        feature = "malicious_relay"
    ))] {
        compile_error!("Only one of `client`, `server`, `relay`, `bootstrap`, or their malicious variants may be enabled at a time.");
    } else if #[cfg(feature = "malicious_client", any(
        feature = "client",
        feature = "server",
        feature = "bootstrap",
        feature = "relay",
        feature = "malicious_bootstrap",
        feature = "malicious_server",
        feature = "malicious_relay"
    ))] {
        compile_error!("Only one of `client`, `server`, `relay`, `bootstrap`, or their malicious variants may be enabled at a time.");
    } else if #[cfg(feature = "malicious_server", any(
        feature = "client",
        feature = "server",
        feature = "bootstrap",
        feature = "relay",
        feature = "malicious_bootstrap",
        feature = "malicious_client",
        feature = "malicious_relay"
    ))] {
        compile_error!("Only one of `client`, `server`, `relay`, `bootstrap`, or their malicious variants may be enabled at a time.");
    } else if #[cfg(feature = "malicious_relay", any(
        feature = "client",
        feature = "server",
        feature = "bootstrap",
        feature = "relay",
        feature = "malicious_bootstrap",
        feature = "malicious_client",
        feature = "malicious_server"
    ))] {
        compile_error!("Only one of `client`, `server`, `relay`, `bootstrap`, or their malicious variants may be enabled at a time.");
    }
);

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
use libp2p::autonat;
use libp2p::futures::StreamExt as _;
use libp2p::relay;
use libp2p::dcutr;
use clap::Parser as _;
use ubyte::ToByteUnit as _;
use num::ToPrimitive as _;

mod config;
mod env_key;
mod grpc;
mod sub_system;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type Swarm = swarm::Swarm<Behaviour>;
type SwarmEvent = swarm::SwarmEvent<BehaviourEvent>;

#[derive(Debug)]
#[derive(derive_more::From)]
struct Event {
    item: Box<dyn std::any::Any + Send>
}

impl Event {
    pub fn new<T>(item: T) -> Self
    where
        T: std::any::Any,
        T: Send,
        T: 'static {
        let item: Box<_> = Box::new(item);
        Self {
            item
        }
    }

    pub fn downcast_ref<T>(&self) -> Option<&T> 
    where
        T: std::any::Any {
        self.item.downcast_ref()
    }

    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: std::any::Any {
        self.item.downcast_mut()
    }

    pub fn downcast<T>(self) -> std::result::Result<T, Self> 
    where
        T: std::any::Any {
        match self.item.downcast::<T>() {
            Ok(item) => {
                let item: T = *item;
                Ok(item)
            },
            Err(item) => {
                let item: Self = Self {
                    item
                };
                Err(item)
            }
        }
    }
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

#[derive(swarm::NetworkBehaviour)]
struct Behaviour {
    #[cfg(any(feature = "relay", feature = "malicious_relay"))]
    pub relay: relay::Behaviour,

    #[cfg(any(
        feature = "client", 
        feature = "server",
        feature = "malicious_client",
        feature = "malicious_server"
    ))]
    pub relay_client: relay::client::Behaviour,
    
    pub autonat: autonat::Behaviour,

    #[cfg(any(
        feature = "client", 
        feature = "server",
        feature = "malicious_client",
        feature = "malicious_server"
    ))]
    pub dcutr: dcutr::Behaviour,

    #[cfg(any(
        feature = "bootstrap",
        feature = "client", 
        feature = "server", 
        feature = "relay",
        feature = "malicious_bootstrap",
        feature = "malicious_client",
        feature = "malicious_server",
        feature = "malicious_relay"
    ))]
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
    
    #[cfg(any(
        feature = "bootstrap",
        feature = "client", 
        feature = "server", 
        feature = "relay",
        feature = "malicious_bootstrap",
        feature = "malicious_client",
        feature = "malicious_server",
        feature = "malicious_relay"
    ))]
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
            let s: std::fmt::Arguments<'_> = format_args!("[{} {}] {}", record_level, record_target, message);
            out.finish(s);
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;

    log::info!("booting");

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
    let protocol_name: libp2p::StreamProtocol = libp2p::StreamProtocol::new("/an");

    #[cfg(any(feature = "bootstrap", feature = "malicious_bootstrap"))]
    let agent_version: String = format!("an-bootstrap/{}", version);

    #[cfg(any(feature = "client", feature = "malicious_client"))]
    let agent_version: String = format!("an-client/{}", version);

    #[cfg(any(feature = "server", feature = "malicious_server"))]
    let agent_version: String = format!("an-server/{}", version);

    #[cfg(any(feature = "relay", feature = "malicious_relay"))]
    let agent_version: String = format!("an-relay/{}", version);

    #[cfg(any(feature = "bootstrap", feature = "malicious_bootstrap"))]
    let identify_cache_size: usize = 50000;

    #[cfg(any(feature = "client", feature = "malicious_client"))]
    let identify_cache_size: usize = if let Some(conf) = &conf
    && let Some(client) = &conf.client
    && let Some(identity_cache_size) = client.identity_cache_size {
        identity_cache_size
    } else {
        1000
    };

    #[cfg(any(feature = "server", feature = "malicious_server"))]
    let identify_cache_size: usize = if let Some(conf) = &conf 
    && let Some(server) = &conf.server 
    && let Some(identity_cache_size) = server.identity_cache_size {
        identity_cache_size
    } else {
        2000
    };

    #[cfg(any(feature = "relay", feature = "malicious_relay"))]
    let identify_cache_size: usize = if let Some(conf) = &conf
    && let Some(relay) = &conf.relay
    && let Some(identity_cache_size) = relay.identity_cache_size {
        identity_cache_size
    } else {
        5000
    };

    #[cfg(any(feature = "bootstrap", feature = "malicious_bootstrap"))]
    let identify_interval: std::time::Duration = std::time::Duration::from_secs(5);

    #[cfg(any(feature = "client", feature = "malicious_client"))]
    let identify_interval: std::time::Duration = std::time::Duration::from_mins(5);

    #[cfg(any(feature = "server", feature = "malicious_server"))]
    let identify_interval: std::time::Duration = std::time::Duration::from_mins(5);

    #[cfg(any(feature = "relay", feature = "malicious_relay"))]
    let identify_interval: std::time::Duration = std::time::Duration::from_mins(5);

    let local_keypair: identity::Keypair = identity::Keypair::generate_ed25519();
    let local_public_key: identity::PublicKey = local_keypair.public();
    let local_peer_id: libp2p::PeerId = local_keypair.public().into();

    log::info!("peer identity initialized: {:?}", local_peer_id);

    let mut quic_config: quic::Config = quic::Config::new(&local_keypair);
    quic_config.handshake_timeout = std::time::Duration::from_millis(3000);
    quic_config.keep_alive_interval = std::time::Duration::from_secs(10);
    quic_config.max_concurrent_stream_limit = 512;
    quic_config.max_connection_data = 10.megabytes().as_u64().to_u32().unwrap();
    quic_config.max_idle_timeout = 60000;
    quic_config.max_stream_data = 1.megabytes().as_u64().to_u32().unwrap();

    #[cfg(any(feature = "bootstrap", feature = "malicious_bootstrap"))]
    let mut swarm: libp2p::Swarm<_> = libp2p::SwarmBuilder::with_existing_identity(local_keypair)
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_behaviour(|_| {
            let kad_store: kad::store::MemoryStore = kad::store::MemoryStore::new(local_peer_id);

            let mut kad_conf: kad::Config = kad::Config::new(protocol_name);
            kad_conf.disjoint_query_paths(true);
            kad_conf.set_caching(kad::Caching::Enabled{ max_peers: 256 });
            kad_conf.set_kbucket_inserts(kad::BucketInserts::Manual);
            kad_conf.set_kbucket_pending_timeout(std::time::Duration::from_mins(1));
            kad_conf.set_kbucket_size(
                128.try_into().expect("non zero")
            );
            kad_conf.set_max_packet_size(
                1.kilobytes().as_u64().to_usize().unwrap()
            );
            kad_conf.set_parallelism(
                32.try_into().expect("non zero")
            );
            kad_conf.set_periodic_bootstrap_interval(Some(std::time::Duration::from_mins(5)));
            kad_conf.set_provider_publication_interval(None);
            kad_conf.set_provider_record_ttl(Some(std::time::Duration::from_hours(72)));
            kad_conf.set_publication_interval(None);
            kad_conf.set_query_timeout(std::time::Duration::from_secs(30));
            kad_conf.set_record_filtering(kad::StoreInserts::FilterBoth);
            kad_conf.set_record_ttl(Some(std::time::Duration::from_hours(72)));
            kad_conf.set_replication_factor(
                256.try_into().expect("non zero")
            );
            kad_conf.set_replication_interval(Some(std::time::Duration::from_hours(1)));
            kad_conf.set_substreams_timeout(std::time::Duration::from_millis(20000));

            let mut kad: kad::Behaviour<_> = kad::Behaviour::with_config(local_peer_id, kad_store, kad_conf);
            
            kad.set_mode(Some(kad::Mode::Server));

            let mut autonat_conf = autonat::Config::default();
            autonat_conf.boot_delay = std::time::Duration::from_secs(1);
            autonat_conf.confidence_max = 3;
            autonat_conf.max_peer_addresses = 10;
            autonat_conf.only_global_ips = false;
            autonat_conf.refresh_interval = std::time::Duration::from_hours(1);
            autonat_conf.retry_interval = std::time::Duration::from_secs(60);
            autonat_conf.throttle_clients_global_max = 1000;
            autonat_conf.throttle_clients_peer_max = 10;
            autonat_conf.throttle_clients_period = std::time::Duration::from_secs(1);
            autonat_conf.throttle_server_period = std::time::Duration::from_secs(30);
            autonat_conf.timeout = std::time::Duration::from_secs(30);
            autonat_conf.use_connected = true;

            let autonat = autonat::Behaviour::new(local_peer_id, autonat_conf);

            let identify_config: identify::Config = identify::Config::new(protocol_version, local_public_key)
                .with_agent_version(agent_version)
                .with_cache_size(identify_cache_size)
                .with_hide_listen_addrs(false)
                .with_interval(identify_interval)
                .with_push_listen_addr_updates(true);

            let identify: identify::Behaviour = identify::Behaviour::new(identify_config);            

            Behaviour {
                autonat,
                kad,
                identify
            }
        })?
        .build();

    #[cfg(any(feature = "client", feature = "malicious_client"))]
    let mut swarm: libp2p::Swarm<_> = libp2p::SwarmBuilder::with_existing_identity(local_keypair)
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|_, relay_client| {
            let kad_store: kad::store::MemoryStore = kad::store::MemoryStore::new(local_peer_id);
            
            let mut kad_conf: kad::Config = kad::Config::new(protocol_name);
            kad_conf.disjoint_query_paths(true);
            kad_conf.set_caching(kad::Caching::Enabled{ max_peers: 64 });
            kad_conf.set_kbucket_inserts(kad::BucketInserts::Manual);
            kad_conf.set_kbucket_pending_timeout(std::time::Duration::from_mins(1));
            kad_conf.set_kbucket_size(kad::K_VALUE);
            kad_conf.set_max_packet_size(
                1.kilobytes().as_u64().to_usize().unwrap()
            );
            kad_conf.set_parallelism(kad::ALPHA_VALUE);
            kad_conf.set_periodic_bootstrap_interval(Some(std::time::Duration::from_mins(5)));
            kad_conf.set_provider_publication_interval(None);
            kad_conf.set_provider_record_ttl(None);
            kad_conf.set_publication_interval(None);
            kad_conf.set_query_timeout(std::time::Duration::from_mins(1));
            kad_conf.set_record_filtering(kad::StoreInserts::FilterBoth);
            kad_conf.set_record_ttl(Some(std::time::Duration::from_hours(48)));
            kad_conf.set_replication_factor(kad::K_VALUE);
            kad_conf.set_replication_interval(None);
            kad_conf.set_substreams_timeout(std::time::Duration::from_secs(10));
            
            let mut kad: kad::Behaviour<_> = kad::Behaviour::with_config(local_peer_id, kad_store, kad_conf);
            
            kad.set_mode(Some(kad::Mode::Client));

            let mut autonat_conf = autonat::Config::default();
            autonat_conf.boot_delay = std::time::Duration::from_secs(1);
            autonat_conf.confidence_max = 3;
            autonat_conf.max_peer_addresses = 5;
            autonat_conf.only_global_ips = false;
            autonat_conf.refresh_interval = std::time::Duration::from_mins(15);
            autonat_conf.retry_interval = std::time::Duration::from_secs(30);
            autonat_conf.throttle_clients_global_max = 0;
            autonat_conf.throttle_clients_peer_max = 0;
            autonat_conf.throttle_clients_period = std::time::Duration::from_secs(60);
            autonat_conf.throttle_server_period = std::time::Duration::from_secs(60);
            autonat_conf.timeout = std::time::Duration::from_secs(15);
            autonat_conf.use_connected = true;

            let autonat = autonat::Behaviour::new(local_peer_id, autonat_conf);

            let dcutr: dcutr::Behaviour = dcutr::Behaviour::new(local_peer_id);
        
            let identify_config: identify::Config = identify::Config::new(protocol_version, local_public_key)
                .with_agent_version(agent_version)
                .with_cache_size(identify_cache_size)
                .with_hide_listen_addrs(true)
                .with_interval(identify_interval)
                .with_push_listen_addr_updates(true);

            let identify: identify::Behaviour = identify::Behaviour::new(identify_config);

            Behaviour {
                relay_client,
                autonat,
                dcutr,
                kad,
                identify
            }
        })?
        .build();

    #[cfg(any(feature = "server", feature = "malicious_server"))] 
    let mut swarm: libp2p::Swarm<_> = libp2p::SwarmBuilder::with_existing_identity(local_keypair)
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|_, relay_client| {
            let kad_store: kad::store::MemoryStore = kad::store::MemoryStore::new(local_peer_id);
            
            let mut kad_conf: kad::Config = kad::Config::new(protocol_name);
            kad_conf.disjoint_query_paths(true);
            kad_conf.set_caching(kad::Caching::Enabled{ max_peers: 256 });
            kad_conf.set_kbucket_inserts(kad::BucketInserts::Manual);
            kad_conf.set_kbucket_pending_timeout(std::time::Duration::from_mins(1));
            kad_conf.set_kbucket_size(kad::K_VALUE);
            kad_conf.set_max_packet_size(
                1.kilobytes().as_u64().to_usize().unwrap()
            );
            kad_conf.set_parallelism(kad::ALPHA_VALUE);
            kad_conf.set_periodic_bootstrap_interval(Some(std::time::Duration::from_mins(5)));
            kad_conf.set_provider_publication_interval(Some(std::time::Duration::from_hours(6)));
            kad_conf.set_provider_record_ttl(Some(std::time::Duration::from_hours(48)));
            kad_conf.set_publication_interval(Some(std::time::Duration::from_hours(24)));
            kad_conf.set_query_timeout(std::time::Duration::from_mins(1));
            kad_conf.set_record_filtering(kad::StoreInserts::FilterBoth);
            kad_conf.set_record_ttl(Some(std::time::Duration::from_hours(48)));
            kad_conf.set_replication_factor(kad::K_VALUE);
            kad_conf.set_replication_interval(None);
            kad_conf.set_substreams_timeout(std::time::Duration::from_secs(10));
            
            let mut kad: kad::Behaviour<_> = kad::Behaviour::with_config(local_peer_id, kad_store, kad_conf);

            kad.set_mode(Some(kad::Mode::Server));

            let mut autonat_conf = autonat::Config::default();
            autonat_conf.boot_delay = std::time::Duration::from_secs(1);
            autonat_conf.confidence_max = 3;
            autonat_conf.max_peer_addresses = 8;
            autonat_conf.only_global_ips = false;
            autonat_conf.refresh_interval = std::time::Duration::from_mins(30);
            autonat_conf.retry_interval = std::time::Duration::from_secs(60);
            autonat_conf.throttle_clients_global_max = 50;
            autonat_conf.throttle_clients_peer_max = 3;
            autonat_conf.throttle_clients_period = std::time::Duration::from_secs(5);
            autonat_conf.throttle_server_period = std::time::Duration::from_secs(30);
            autonat_conf.timeout = std::time::Duration::from_secs(30);
            autonat_conf.use_connected = true;

            let autonat = autonat::Behaviour::new(local_peer_id, autonat_conf);

            let dcutr: dcutr::Behaviour = dcutr::Behaviour::new(local_peer_id);

            let identify_config: identify::Config = identify::Config::new(protocol_version, local_public_key)
                .with_agent_version(agent_version)
                .with_cache_size(identify_cache_size)
                .with_hide_listen_addrs(false)
                .with_interval(identify_interval)
                .with_push_listen_addr_updates(true);

            let identify: identify::Behaviour = identify::Behaviour::new(identify_config);

            Behaviour {
                relay_client,
                autonat,
                dcutr,
                kad,
                identify
            }
        })?
        .build();
    
    #[cfg(any(feature = "relay", feature = "malicious_relay"))]
    let mut swarm: libp2p::Swarm<_> = libp2p::SwarmBuilder::with_existing_identity(local_keypair)
        .with_tokio()
        .with_quic_config(|_| quic_config)
        .with_behaviour(|_| {
            let relay_config: relay::Config = relay::Config {
                max_circuit_bytes: 1.mebibytes().as_u64(),
                max_circuit_duration: std::time::Duration::from_secs(300),
                max_reservations: 512,
                max_reservations_per_peer: 2,
                max_circuits: 1024,
                max_circuits_per_peer: 4,
                reservation_duration: std::time::Duration::from_hours(1),
                reservation_rate_limiters: vec![],
                circuit_src_rate_limiters: vec![]
            };
            let relay: relay::Behaviour = relay::Behaviour::new(local_peer_id, relay_config);

            let kad_store: kad::store::MemoryStore = kad::store::MemoryStore::new(local_peer_id);

            let mut kad_conf: kad::Config = kad::Config::new(protocol_name);
            kad_conf.disjoint_query_paths(true);
            kad_conf.set_caching(kad::Caching::Enabled{ max_peers: 128 });
            kad_conf.set_kbucket_inserts(kad::BucketInserts::Manual);
            kad_conf.set_kbucket_pending_timeout(std::time::Duration::from_millis(60000));
            kad_conf.set_kbucket_size(
                64.try_into().expect("non zero")
            );
            kad_conf.set_max_packet_size(
                1.kilobytes().as_u64().to_usize().unwrap()
            );
            kad_conf.set_parallelism(
                16.try_into().expect("non zero")
            );
            kad_conf.set_periodic_bootstrap_interval(Some(std::time::Duration::from_mins(5)));
            kad_conf.set_provider_publication_interval(None);
            kad_conf.set_provider_record_ttl(None);
            kad_conf.set_publication_interval(None);
            kad_conf.set_query_timeout(std::time::Duration::from_mins(1));
            kad_conf.set_record_filtering(kad::StoreInserts::FilterBoth);
            kad_conf.set_record_ttl(Some(std::time::Duration::from_hours(24)));
            kad_conf.set_replication_factor(
                64.try_into().expect("non zero")
            );
            kad_conf.set_replication_interval(Some(std::time::Duration::from_hours(2)));
            kad_conf.set_substreams_timeout(std::time::Duration::from_secs(10));

            let mut kad: kad::Behaviour<_> = kad::Behaviour::with_config(local_peer_id, kad_store, kad_conf);
            
            kad.set_mode(Some(kad::Mode::Server));

            let mut autonat_conf = autonat::Config::default();
            autonat_conf.boot_delay = std::time::Duration::from_secs(1);
            autonat_conf.confidence_max = 3;
            autonat_conf.max_peer_addresses = 10;
            autonat_conf.only_global_ips = false;
            autonat_conf.refresh_interval = std::time::Duration::from_hours(1);
            autonat_conf.retry_interval = std::time::Duration::from_secs(60);
            autonat_conf.throttle_clients_global_max = 1000;
            autonat_conf.throttle_clients_peer_max = 10;
            autonat_conf.throttle_clients_period = std::time::Duration::from_secs(1);
            autonat_conf.throttle_server_period = std::time::Duration::from_secs(30);
            autonat_conf.timeout = std::time::Duration::from_secs(30);
            autonat_conf.use_connected = true;

            let autonat = autonat::Behaviour::new(local_peer_id, autonat_conf);
            
            let identify_config: identify::Config = identify::Config::new(protocol_version, local_public_key)
                .with_agent_version(agent_version)
                .with_cache_size(identify_cache_size)
                .with_hide_listen_addrs(false)
                .with_interval(identify_interval)
                .with_push_listen_addr_updates(true);

            let identify: identify::Behaviour = identify::Behaviour::new(identify_config);

            Behaviour {
                relay,
                autonat,
                kad,
                identify
            }
        })
        .expect("")
        .build();

    swarm.listen_on("/ip4/0.0.0.0/udp/4001/quic-v1".parse()?)?;

    #[cfg(any(feature = "server", feature = "malicious_server"))] {
        for addr in &dial {
            if addr.to_string().contains("p2p") {
                let is_p2p: bool = addr.iter().any(|protocol| matches!(protocol, libp2p::multiaddr::Protocol::P2p(_)));
    
                if is_p2p {
                    let circuit_addr: libp2p::Multiaddr = addr.clone().with(libp2p::multiaddr::Protocol::P2pCircuit);
                    
                    log::info!("server attempting relay reservation: {}", circuit_addr);

                    // these are speculative attempts
                    swarm.listen_on(circuit_addr).ok();
                }
            }
        }
    }

    let (sx, mut rx) = tokio::sync::mpsc::channel::<Event>(1000);

    let grpc_endpoint: std::net::SocketAddr = if let Some(grpc_endpoint) = cli.grpc_endpoint {
        grpc_endpoint
    } else if let Some(conf) = &conf && let Some(grpc) = &conf.grpc_endpoint {
        grpc.to_owned()
    } else {
        "0.0.0.0:8080".parse()?
    };

    let grpc_server: grpc::Server = grpc::Server::new(sx);
    let grpc_server: grpc::proto::node_server::NodeServer<_> = grpc::proto::node_server::NodeServer::new(grpc_server);
    let grpc = tonic::transport::Server::builder()
        .add_service(grpc_server)
        .serve(grpc_endpoint);

    let ctrl_c = tokio::signal::ctrl_c();

    tokio::pin!(grpc);
    tokio::pin!(ctrl_c);

    let bootstrap: sub_system::bootstrap::Bootstrap = sub_system::bootstrap::Bootstrap::builder()
        .cooldown(std::time::Duration::from_secs(16))
        .timeout_duration(std::time::Duration::from_secs(8))
        .min_peers(2)
        .addrs(dial)
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

    let discovery_monitor: sub_system::discovery_monitor::DiscoveryMonitor = sub_system::discovery_monitor::DiscoveryMonitor::builder()
        .interval(std::time::Duration::from_secs(5))
        .build();

    let mut sub_system_bus: sub_system::Bus = sub_system::Bus::default();
    sub_system_bus.add_system(bootstrap);
    // sub_system_bus.add_system(connection_manager);
    sub_system_bus.add_system(routing_monitor);
    sub_system_bus.add_system(discovery_monitor);
    sub_system_bus.add_system(sub_system::dialer::Dialer);
    sub_system_bus.add_system(sub_system::metadata::Metadata);
    sub_system_bus.add_system(sub_system::monitor::Monitor);

    cfg_if::cfg_if!(
        if #[cfg(feature = "malicious_relay")] {
            let identity_spoofer: sub_system::identity_spoofer::IdentitySpoofer = sub_system::identity_spoofer::IdentitySpoofer::builder()
                .interval(std::time::Duration::from_secs(30))
                .build();

            let slug: sub_system::slug::Slug = sub_system::slug::Slug::builder()
                .delay(std::time::Duration::from_secs(30))
                .build();

            sub_system_bus.add_system(sub_system::dht_poison::DhtPoison);
            sub_system_bus.add_system(sub_system::relay_killer::RelayKiller);
            // sub_system_bus.add_system(sub_system::self_destruct::SelfDestruct);
            sub_system_bus.add_system(identity_spoofer);
            sub_system_bus.add_system(slug);
        }
    );

    log::info!("finished booting, entering event loop");

    loop {
        tokio::select!(
            _ = &mut ctrl_c => {
                break
            },
            _ = &mut grpc => {
                break
            },
            event = swarm.select_next_some() => {
                sub_system_bus.receive(&mut swarm, Event::new(event))
            },
            Some(event) = rx.recv() => {
                sub_system_bus.receive(&mut swarm, event)
            }
        );
    }

    tokio::signal::ctrl_c().await?;
    Ok(())
}