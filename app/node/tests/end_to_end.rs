#![allow(clippy::vec_init_then_push)]
#![allow(unused)]
#![cfg(feature = "end_to_end")]

use std::io::Read as _;
use std::num::NonZeroI128;
use std::thread::spawn;
use futures_util::StreamExt as _;
use futures_util::TryStreamExt as _;
use testcontainers::ImageExt;
use testcontainers::ImageExt as _;
use testcontainers::runners::AsyncRunner as _;
use tokio::io::AsyncWriteExt;

mod proto {
    include!("../proto_target/an.rs");
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod log {
    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug)]
    #[derive(strum::EnumCount)]
    #[derive(thiserror::Error)]
    pub enum Error {
        #[error("{0}")]
        Io(#[from] std::io::Error),
        #[error("unable to parse")]
        #[strum(serialize = "unparsable")]
        #[strum(serialize = "Unparsable")]
        #[strum(serialize = "UNPARSABLE")]
        Unparsable,
        #[error("not a log file")]
        #[strum(serialize = "not_log_file")]
        NotLogFile,
        #[error("")]
        FileNotFound
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(Copy)]
    #[derive(PartialEq)]
    #[derive(Eq)]
    #[derive(Default)]
    #[derive(strum::EnumCount)]
    #[derive(strum::EnumIter)]
    #[derive(strum::EnumString)]
    pub enum Severity {
        #[default]
        #[strum(serialize = "INFO")]
        Info,
        #[strum(serialize = "WARN")]
        Warn,
        #[strum(serialize = "ERROR")]
        Error,
        #[strum(serialize = "DEBUG")]
        Debug
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(PartialEq)]
    #[derive(Eq)]
    #[derive(getset::Getters)]
    #[derive(getset::CopyGetters)]
    #[derive(bon::Builder)]
    pub struct Log {
        #[getset(get_copy = "pub")]
        #[builder(into)]
        #[builder(default = chrono::Utc::now())]
        timestamp: chrono::DateTime<chrono::Utc>,
        #[getset(get = "pub")]
        #[builder(into)]
        #[builder(default = Severity::Info)]
        severity: Severity,
        #[getset(get = "pub")]
        #[builder(into)]
        component: String,
        #[getset(get = "pub")]
        #[builder(into)]
        message: String
    }

    impl std::str::FromStr for Log {
        type Err = Error;

        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            let (timestamp, more) = s.split_once(' ').ok_or(Error::Unparsable)?;
            let timestamp: chrono::DateTime<chrono::Utc> = timestamp.parse().ok().ok_or(Error::Unparsable)?;
            let lb: usize = more.find('[').ok_or(Error::Unparsable)?;
            let rb: usize = more.find(']').ok_or(Error::Unparsable)?;
            let meta: &str = &more[lb + 1..rb];
            let (severity, component) = meta.split_once(' ').ok_or(Error::Unparsable)?;
            let message: &str = more[rb + 1..].trim();
            let message: String = message.to_owned();
            let severity: Severity = severity
                .parse()
                .ok()
                .ok_or(Error::Unparsable)?;
            let component: String = component.to_owned();
            let new: Self = Self {
                timestamp,
                severity,
                component,
                message
            };
            Ok(new)
        }
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(PartialEq)]
    #[derive(Eq)]
    pub struct Report {
        id_to_logs: std::collections::HashMap<String, Vec<Log>>
    }

    impl Report {
        pub fn from_log_file_paths(paths: &[std::path::PathBuf]) -> Result<Self> {
            let mut id_to_logs: std::collections::HashMap<String, Vec<Log>> = std::collections::HashMap::new();
            for path in paths {
                let file_name: std::ffi::OsString = path.file_name().ok_or(Error::FileNotFound)?.to_owned();
                let file_name: String = file_name
                    .to_string_lossy()
                    .to_string();
                let (file_name, extension) = file_name.split_once('.').ok_or(Error::Unparsable)?;
                if extension != "log" {
                    return Err(Error::NotLogFile)
                }
                let file_name: String = file_name.to_owned();
                let content: String = std::fs::read_to_string(path)?;
                let mut logs: Vec<_> = vec![];
                for line in content.lines() {
                    let log: Log = line.parse()?;
                    logs.push(log);
                }
                id_to_logs.insert(file_name, logs);
            }
            let new: Self = Self {
                id_to_logs
            };
            Ok(new)
        }

        pub fn from_dir(dir: &std::path::Path) -> Result<Self> {
            let mut paths = vec![];
            for item in std::fs::read_dir(dir)? {
                let item: std::fs::DirEntry = item?;
                let item_path: std::path::PathBuf = item.path();
                let item_file_type: std::fs::FileType = item.file_type()?;
                if item_file_type.is_file() {
                    paths.push(item_path);
                }
            }
            Self::from_log_file_paths(&paths)
        }
    }

    impl Report {
        /// # Proof of Startup
        /// Every node starts successfully.
        pub fn is_proof_of_startup(&self) -> bool {
            self.id_to_logs.values().all(|logs| {
                let mut has_boot: bool = false;
                let mut has_id: bool = false;
                let mut has_looped: bool = false;
                for log in logs {
                    let msg: &str = log.message();
                    if msg.contains("booting") {
                        has_boot = true;
                    }
                    if msg.contains("peer identity initialized") {
                        has_id = true;
                    }
                    if msg.contains("entering event loop") {
                        has_looped = true;
                    }
                }
                has_boot && has_id && has_looped
            })
        }

        pub fn is_proof_of_relay_usage(&self) -> bool {
            self.id_to_logs.values().any(|logs| {
                logs.iter().any(|log| {
                    let msg: String = log.message().to_lowercase();
                    let has_relay: bool = msg.contains("relay");
                    let has_dial: bool = msg.contains("dial");
                    let has_forward: bool = msg.contains("forward");
                    let has_circuit: bool = msg.contains("circuit");
                    has_relay && (has_dial || has_forward || has_circuit)
                })
            })
        }

        /// # Proof of GRPC Interaction
        pub fn is_proof_of_grpc_interaction(&self) -> bool {
            self.id_to_logs.values().any(|logs| {
                logs.iter().any(|log| {
                    log.component().contains("grpc") && log.message().contains("dial request")
                })
            })
        }

        pub fn is_proof_of_routing_table_population(&self) -> bool {
            self.id_to_logs.values().all(|logs| {
                logs.iter().any(|log| {
                    log.message().contains("knows about") && log.message().contains("peers")
                })
            })
        }

        pub fn is_proof_of_unique_identity(&self) -> bool {
            let mut seen: std::collections::HashMap<&str, bool> = std::collections::HashMap::new();
            for (_, logs) in self.id_to_logs.iter() {
                for log in logs {
                    let msg: &str = log.message();
                    if msg.contains("PeerId")
                    && let Some(from) = msg.find('(')
                    && let Some(to) = msg.find(')') {
                        let peer_id: &str = &msg[from..to];
                        if seen.insert(peer_id, true).is_some() {
                            return false
                        }
                    }
                }
            }
            true
        }

        pub fn is_proof_of_sybil_attack(&self) -> bool {
            // placeholder
            self.id_to_logs.values().any(|logs| {
                logs.iter().any(|log| {
                    let msg = log.message().to_lowercase();
                    msg.contains("sybil")
                    || msg.contains("malicious")
                    || msg.contains("routing pollution")
                    || msg.contains("invalid peer")
                })
            })
        }

        pub fn is_proof_of_eventual_discovery(&self) -> bool {
            self.id_to_logs.values().all(|logs| {
                logs.iter().any(|line| {
                    line.message().contains("knows about") && !line.message().contains("knows no one")
                })
            })
        }
    
        pub fn is_proof_of_successful_bootstrap(&self) -> bool {
            self.id_to_logs.values().any(|logs| {
                logs.iter().any(|line| {
                    line.message().contains("bootstrap complete, remaining: 0")
                })
            })
        }

        pub fn is_proof_of_causality(
            &self,
            lhs_container_id: &str,
            lhs_log_message: &str,
            rhs_container_id: &str,
            rhs_log_message: &str
        ) -> bool {
            let lhs_time = self.id_to_logs[lhs_container_id].iter()
                .find(|line| {
                    line.message.contains(lhs_log_message)
                })
                .map(|line| {
                    line.timestamp
                });
            let Some(lhs_time) = lhs_time else {
                return false
            };
            let rhs_time = self.id_to_logs[rhs_container_id].iter()
                .find(|line| {
                    line.message.contains(rhs_log_message)
                })
                .map(|line| {
                    line.timestamp
                });
            let Some(rhs_time) = rhs_time else {
                return false
            };
            lhs_time < rhs_time
        }

        pub fn is_proof_of_convergence(&self, expected_peer_count: usize) -> bool {
            for (_, logs) in self.id_to_logs.iter() {
                let reached: bool = logs.iter().any(|line| {
                    line.message.contains("knows about") && line.message.contains(&format!("{} peers", expected_peer_count))
                });
                if !reached {
                    return false
                }
            }
            true
        }

        /// # Proof of Connectivity Persistence
        /// Once the expected peer count is reached, it never drops again.
        pub fn is_proof_of_connectivity_persistence(&self, expected_count: usize) -> bool {
            for logs in self.id_to_logs.values() {
                let mut converged: bool = false;
                for line in logs {
                    if line.message().contains("knows about") && line.message.contains(&format!("{} peers", expected_count)) {
                        converged = true;
                    }
                    if converged && line.message.contains("peer count dropped") {
                        return false
                    }
                }
            }
            true
        }

        /// # Proof of Cohesion
        /// There are no partitioned nodes and the network's topology is connected.
        pub fn is_proof_of_cohesion(&self) -> bool {
            use std::collections::{HashMap, HashSet, VecDeque};
            let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

            for logs in self.id_to_logs.values() {
                let mut peer_id: Option<String> = None;

                for log in logs {
                    if !log.message().contains("PeerId") {
                        continue;
                    }

                    let msg = log.message();
                    let Some(start) = msg.find('(') else { continue };
                    let Some(end) = msg.find(')') else { continue };

                    peer_id = Some(msg[start + 1..end].to_string());
                }

                let Some(peer_id) = peer_id else { continue };

                let Some(log) = logs.iter().rfind(|log| log.message().contains("knows about")) else {
                    continue;
                };

                let msg = log.message();

                let Some(start) = msg.find('[') else { continue };
                let Some(end) = msg.find(']') else { continue };

                let peer_ids: Vec<String> = msg[start + 1..end]
                    .split(',')
                    .map(|id| id.trim().to_string())
                    .collect();

                for p in &peer_ids {
                    graph.entry(peer_id.clone()).or_default().insert(p.clone());
                    graph.entry(p.clone()).or_default().insert(peer_id.clone());
                }
            }

            if graph.is_empty() {
                return false;
            }

            // BFS to check connectivity
            let start = graph.keys().next().unwrap().clone();
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();

            queue.push_back(start.clone());
            visited.insert(start);

            while let Some(node) = queue.pop_front() {
                if let Some(neighbors) = graph.get(&node) {
                    for n in neighbors {
                        if visited.insert(n.clone()) {
                            queue.push_back(n.clone());
                        }
                    }
                }
            }

            visited.len() == graph.len()
        }

        /// # Proof of Stability
        /// Churn decreases over time or is below the threshold after the initial discovery phase.
        pub fn is_proof_of_stability(&self, churn_threshold: usize) -> bool {
            for (_, logs) in self.id_to_logs.iter() {
                let churn_vals: Vec<_> = logs
                    .iter()
                    .filter(|log| {
                        log.message().contains("routing churn:")
                    })
                    .filter_map(|log| {
                        log.message()
                            .split("routing churn: ")
                            .nth(1)?
                            .parse::<usize>()
                            .ok()
                    })
                    .collect();
                if let Some(&last_churn) = churn_vals.last() && last_churn > churn_threshold {
                    return false
                }
            }
            true
        }

        pub fn is_proof_of_propagation(&self, origin_id: &str, msg: &str) -> bool {
            let sent: bool = self.id_to_logs
                .get(origin_id)
                .map(|logs| {
                    logs.iter().any(|log| {
                        log.message().contains("broadcast") && log.message().contains(msg)
                    })
                })
                .unwrap_or(false);
            if !sent {
                return false
            }
            self.id_to_logs
                .iter()
                .filter(|(id, _)| {
                    *id != origin_id
                })
                .all(|(_, logs)| {
                    logs.iter().any(|log| {
                        log.message().contains("received") && log.message().contains(msg)
                    })
                })
        }
    }
}

mod network {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    pub const A: &str = "an_a";
    pub const B: &str = "an_b";
    pub const C: &str = "an_c";

    #[derive(getset::Getters)]
    pub struct Network<'a> {
        docker: &'a bollard::Docker,
        #[getset(get = "pub")]
        name: String
    }

    #[bon::bon]
    impl<'a> Network<'a> {
        #[builder]
        #[builder(finish_fn = "reserve")]
        pub async fn new(
            docker: &'a bollard::Docker,
            docker_network_conf: bollard::secret::NetworkCreateRequest,
            #[builder(into)]
            name: String
        ) -> Result<Self> {
            docker.create_network(docker_network_conf).await?;
            let new: Self = Self {
                docker,
                name
            };
            Ok(new)
        }
    }

    impl<'a> Network<'a> {
        pub async fn release(self) {
            let containers: Vec<_> = self.docker.list_containers(None).await.unwrap_or_default();
            for container in containers {
                let Some(id) = container.id else {
                    continue
                };
                let request: bollard::secret::NetworkDisconnectRequest = bollard::secret::NetworkDisconnectRequest {
                    container: id,
                    force: Some(true)
                };
                self.docker.disconnect_network(&self.name, request).await.ok();
            }
            self.docker.remove_network(&self.name).await.ok();
        }
    }
}

type Container = testcontainers::ContainerAsync<testcontainers::GenericImage>;

trait Docker {
    async fn load(&self, path: &std::path::Path) -> Result<()>;
    async fn load_built_tar_image_from_ws_target_dir(&self) -> Result<()>;
    async fn reset(&self) -> Result<()>;
    async fn reset_network(&self, network_name: &str) -> Result<()>;
    async fn write_logs_to_file(&self, out_dir: &std::path::Path, containers: Vec<Container>) -> Result<()>;
}

impl Docker for bollard::Docker {
    async fn load(&self, path: &std::path::Path) -> Result<()> {
        let mut file = std::fs::File::open(path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();
        let options = bollard::query_parameters::ImportImageOptions {
            quiet: false,
            ..Default::default()
        };
        let mut stream = self.import_image(options, bollard::body_full(bytes.into()), None);
        while let Some(_) = stream.try_next().await.unwrap() {

        }
        Ok(())
    }

    async fn load_built_tar_image_from_ws_target_dir(&self) -> Result<()> {
        std::process::Command::new("cargo")
            .arg("run")
            .arg("--package")
            .arg("task")
            .arg("build-image")
            .spawn()
            .expect("failed to build image")
            .wait()
            .expect("failed to build image");
        
        let ws_dir: std::path::PathBuf = cargo_metadata::MetadataCommand::new()
            .exec()
            .unwrap()
            .workspace_root
            .to_string()
            .into();
        
        let image_dir: std::path::PathBuf = ws_dir
            .join("target")
            .join("image");
        
        let image_path: std::path::PathBuf = image_dir.join("node.tar");
        
        self.load(&image_path).await;
        Ok(())
    }

    async fn reset(&self) -> Result<()> {
        let containers: Vec<_> = self.list_containers(None).await?;
        for container in containers {
            self.stop_container(&container.id.to_owned().unwrap(), None).await?;
        }
        Ok(())
    }

    async fn reset_network(&self, network_name: &str) -> Result<()> {
        let containers: Vec<_> = self.list_containers(None).await?;
        for container in containers {
            let Some(id) = container.id else {
                continue
            };
            let request: bollard::secret::NetworkDisconnectRequest = bollard::secret::NetworkDisconnectRequest {
                container: id,
                force: Some(true)
            };
            self.disconnect_network(network_name, request).await?;
        }
        Ok(())   
    }

    async fn write_logs_to_file(&self, out_dir: &std::path::Path, containers: Vec<testcontainers::ContainerAsync<testcontainers::GenericImage>>) -> Result<()> {
        std::fs::remove_dir_all(out_dir).ok();
        std::fs::create_dir_all(out_dir)?;
        let logs_conf: bollard::query_parameters::LogsOptions = bollard::query_parameters::LogsOptions {
            stdout: true,
            stderr: true,
            timestamps: true,
            tail: "all".into(),
            ..Default::default()
        };
        for container in containers {
            let logs_conf: bollard::query_parameters::LogsOptions = logs_conf.to_owned();
            let container_id: &str = container.id();
            let mut container_path: std::path::PathBuf = out_dir.join(container_id);
            container_path.set_extension("log");
            let mut file: tokio::fs::File = tokio::fs::File::create(container_path).await.unwrap();
            let mut stream = self.logs(container_id, Some(logs_conf));
            while let Some(log) = stream.next().await {
                let log: bollard::container::LogOutput = log?;
                let bytes = match log {
                    bollard::container::LogOutput::StdOut {
                        message
                    } => {
                        message
                    },
                    bollard::container::LogOutput::StdErr {
                        message
                    } => {
                        message
                    },
                    bollard::container::LogOutput::Console {
                        message
                    } => {
                        message
                    },
                    _ => continue
                };
                file.write_all(&bytes).await.unwrap()
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
trait Test {
    async fn run(&self, docker: &bollard::Docker);
}

struct Harness {
    docker: bollard::Docker,
    tests: Vec<Box<dyn Test>>
}

impl Harness {
    pub fn new(docker: bollard::Docker) -> Self {
        let tests: Vec<_> = vec![];
        Self {
            docker,
            tests
        }
    }
}

impl Harness {
    pub fn add_test<T>(&mut self, test: T)
    where
        T: Test,
        T: 'static {
        self.tests.push(Box::new(test));
    }

    pub async fn launch(self) {
        self.docker.reset().await.ok();
        self.docker.load_built_tar_image_from_ws_target_dir().await.unwrap();
        for test in self.tests.iter() {
            test.run(&self.docker).await;
            self.docker.reset().await.ok();
        }
    }
}

struct NatEasy;

#[async_trait::async_trait]
impl Test for NatEasy {
    async fn run(&self, docker: &bollard::Docker) {
        let log_dir: std::path::PathBuf = std::path::PathBuf::new()
            .join("tests")
            .join("log")
            .join("nat_easy");

        let network_a_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::A.to_owned(),
            driver: None,
            scope: None,
            internal: None,
            attachable: None,
            ingress: None,
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: None,
            enable_ipv6: None,
            options: None,
            labels: None
        };

        let network_b_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::B.to_owned(),
            driver: None,
            scope: None,
            internal: None,
            attachable: None,
            ingress: None,
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: None,
            enable_ipv6: None,
            options: None,
            labels: None
        };

        let network_a: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_a_conf)
            .name(network::A)
            .reserve()
            .await
            .unwrap();
    
        let network_b: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_b_conf)
            .name(network::B)
            .reserve()
            .await
            .unwrap();

        let udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let tcp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

        let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./bootstrap"])
            .with_network(network_a.name())
            .with_network(network_b.name())
            .start()
            .await
            .unwrap();

        let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.expect("bridge ip addr");
        let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

        let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./relay", "--dial", &bootstrap_addr])
            .with_network(network_a.name())
            .with_network(network_b.name())
            .start()
            .await
            .expect("successful container launch");

        let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./server", "--dial", &bootstrap_addr])
            .with_network(network_b.name())
            .start()
            .await
            .expect("successful container launch");

        let server_ip: std::net::IpAddr = server.get_bridge_ip_address().await.expect("bridge ip addr");
        let server_addr: String = format!("/ip4/{}/udp/4001/quic-v1", server_ip);

        let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./client", "--dial", &bootstrap_addr])
            .with_network(network_a.name())
            .start()
            .await
            .expect("successful container launch");

        let client_grpc_port: u16 = client.get_host_port_ipv4(8080).await.expect("host port ipv4");
        let client_gprc_endpoint: String = format!("http://127.0.0.1:{}", client_grpc_port);
        let mut client_grpc: proto::node_client::NodeClient<_> = wait_for_grpc(client_gprc_endpoint).await;

        let client_request: proto::DialRequest = proto::DialRequest {
            addr: server_addr    
        };

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        client_grpc.dial(client_request).await.expect("successful dial");
        
        tokio::time::sleep(std::time::Duration::from_mins(1)).await;

        let mut logged: Vec<_> = vec![];
        logged.push(bootstrap);
        logged.push(relay);
        logged.push(server);
        logged.push(client);

        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

        network_a.release().await;
        network_b.release().await;

        docker.write_logs_to_file(&log_dir, logged).await.unwrap();

        let report: log::Report = log::Report::from_dir(&log_dir).unwrap();
        assert!(report.is_proof_of_startup());
        assert!(report.is_proof_of_cohesion());
        assert!(report.is_proof_of_connectivity_persistence(2));
        assert!(report.is_proof_of_grpc_interaction());
    }
}

struct NatHard;

#[async_trait::async_trait]
impl Test for NatHard {
    async fn run(&self, docker: &bollard::Docker) {
        let log_dir: std::path::PathBuf = std::path::PathBuf::new()
            .join("tests")
            .join("log")
            .join("nat_hard");

        let network_a_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::A.to_owned(),
            driver: None,
            scope: None,
            internal: Some(false),
            attachable: Some(false),
            ingress: Some(false),
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: Some(true),
            enable_ipv6: Some(false),
            options: None,
            labels: None
        };

        let network_b_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::B.to_owned(),
            driver: None,
            scope: None,
            internal: Some(false),
            attachable: Some(false),
            ingress: Some(false),
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: Some(true),
            enable_ipv6: Some(false),
            options: None,
            labels: None
        };

        let network_a: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_a_conf)
            .name(network::A)
            .reserve()
            .await
            .unwrap();
    
        let network_b: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_b_conf)
            .name(network::B)
            .reserve()
            .await
            .unwrap();

        let udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let tcp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

        let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./bootstrap"])
            .with_network(network::A)
            .with_network(network::B)
            .start()
            .await
            .expect("successful container launch");

        let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.expect("bridge ip addr");
        let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

        let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./relay", "--dial", &bootstrap_addr])
            .with_network(network::A)
            .with_network(network::B)
            .start()
            .await
            .expect("successful container launch");

        let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./server", "--dial", &bootstrap_addr])
            .with_network(network::B)
            .start()
            .await
            .expect("successful container launch");

        let server_ip: std::net::IpAddr = server.get_bridge_ip_address().await.expect("bridge ip addr");
        let server_addr: String = format!("/ip4/{}/udp/4001/quic-v1", server_ip);

        let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./client", "--dial", &bootstrap_addr])
            .with_network(network::A)
            .start()
            .await
            .expect("successful container launch");

        let client_grpc_port: u16 = client.get_host_port_ipv4(8080).await.expect("host port ipv4");
        let client_gprc_endpoint: String = format!("http://127.0.0.1:{}", client_grpc_port);
        let mut client_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(client_gprc_endpoint).await.expect("successful grpc client");

        let client_request: proto::DialRequest = proto::DialRequest {
            addr: server_addr    
        };

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        client_grpc.dial(client_request).await.expect("successful dial");

        tokio::time::sleep(std::time::Duration::from_mins(1)).await;

        let mut logged: Vec<_> = vec![];
        logged.push(bootstrap);
        logged.push(relay);
        logged.push(server);
        logged.push(client);

        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

        network_a.release().await;
        network_b.release().await;

        docker.write_logs_to_file(&log_dir, logged).await.unwrap();

        let report: log::Report = log::Report::from_dir(&log_dir).unwrap();
        assert!(report.is_proof_of_startup());
        assert!(report.is_proof_of_cohesion());
        assert!(report.is_proof_of_connectivity_persistence(2));
        assert!(report.is_proof_of_grpc_interaction());
    }
}

struct Discovery;

#[async_trait::async_trait]
impl Test for Discovery {
    async fn run(&self, docker: &bollard::Docker) {
        let log_dir: std::path::PathBuf = std::path::PathBuf::new()
            .join("tests")
            .join("log")
            .join("discovery");

        let network: &str = "an";
        let network_udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let network_tpc_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

        docker.reset().await.ok();
        docker.reset_network(network).await.ok();

        let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(network_udp_port)
            .with_exposed_port(network_tpc_port)
            .with_cmd(["./bootstrap"])
            .with_network(network)
            .start()
            .await
            .unwrap();
        let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.unwrap();
        let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

        let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(network_udp_port)
            .with_exposed_port(network_tpc_port)
            .with_cmd(["./relay", "--dial", &bootstrap_addr])
            .with_network(network)
            .start()
            .await
            .unwrap();

        let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(network_udp_port)
            .with_exposed_port(network_tpc_port)
            .with_cmd(["./server", "--dial", &bootstrap_addr])
            .with_network(network)
            .start()
            .await
            .unwrap();

        let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(network_udp_port)
            .with_exposed_port(network_tpc_port)
            .with_cmd(["./client", "--dial", &bootstrap_addr])
            .with_network(network)
            .start()
            .await
            .unwrap();

        let containers: Vec<_> = vec![
            bootstrap,
            relay,
            server,
            client
        ];

        tokio::time::sleep(std::time::Duration::from_mins(9)).await;
        
        let port = containers
            .get(3)
            .unwrap()
            .get_host_port_ipv4(8080)
            .await
            .unwrap();
        let endpoint = format!("http://127.0.0.1:{}", port);
        let mut client = proto::node_client::NodeClient::connect(endpoint).await.unwrap();
        
        let request = tonic::Request::new(proto::PingRequest{ msg: "Hello".to_owned() });
        let response = client.ping(request).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_mins(1)).await;

        docker.reset().await.ok();
        docker.reset_network(network).await.ok();

        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

        docker.write_logs_to_file(&log_dir, containers).await.unwrap();
    }
}
 
struct Simulation;

#[async_trait::async_trait]
impl Test for Simulation {
    async fn run(&self, docker: &bollard::Docker) {
        let log_dir: std::path::PathBuf = std::path::PathBuf::new()
            .join("tests")
            .join("log")
            .join("sim");
        
        let mut containers: Vec<_> = vec![];

        let network_a_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::A.to_owned(),
            driver: None,
            scope: None,
            internal: Some(false),
            attachable: Some(false),
            ingress: Some(false),
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: Some(true),
            enable_ipv6: Some(false),
            options: None,
            labels: None
        };

        let network_b_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::B.to_owned(),
            driver: None,
            scope: None,
            internal: Some(false),
            attachable: Some(false),
            ingress: Some(false),
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: Some(true),
            enable_ipv6: Some(false),
            options: None,
            labels: None
        };

        let network_a: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_a_conf)
            .name(network::A)
            .reserve()
            .await
            .unwrap();
    
        let network_b: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_b_conf)
            .name(network::B)
            .reserve()
            .await
            .unwrap();
        
        let udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let tcp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

        let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./bootstrap"])
            .with_network(network::A)
            .with_network(network::B)
            .start()
            .await
            .expect("successful container launch");

        let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.expect("bridge ip addr");
        let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

        let bootstrap_container_id: &str = bootstrap.id();

        let bootstrap_grpc_port: u16 = bootstrap.get_host_port_ipv4(8080).await.unwrap();
        let bootstrap_grpc_endpoint: String = format!("http://127.0.0.1:{}", bootstrap_grpc_port);
        
        let mut bootstrap_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(bootstrap_grpc_endpoint).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let bootstrap_peer_id_request: proto::PeerIdRequest = proto::PeerIdRequest { };
        let bootstrap_peer_id: String = bootstrap_grpc
            .peer_id(bootstrap_peer_id_request)
            .await
            .unwrap()
            .into_inner()
            .peer_id;

        let bootstrap_addr_a: String = format!("/dns4/bootstrap/ip4/0.0.0.0/udp/4001/quic-v1/p2p/{}", bootstrap_peer_id);
        let bootstrap_addr_b: String = format!("/dns4/bootstrap/ip4/0.0.0.0/udp/4001/quic-v1/p2p/{}", bootstrap_peer_id);

        containers.push(bootstrap);

        for _ in 0..=16 {
            let cmd: &str = if rand::random::<f32>() < 0.25 {
                "./malicious_relay"
            } else {
                "./relay"
            };

            let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
                .with_exposed_port(udp_port)
                .with_exposed_port(tcp_port)
                .with_cmd([cmd, "--dial", &bootstrap_addr])
                .with_network(network::A)
                .with_network(network::B)
                .start()
                .await
                .expect("successful container launch");

            containers.push(relay);
        }

        let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./server", "--dial", &bootstrap_addr])
            .with_network(network::B)
            .start()
            .await
            .expect("successful container launch");

        let server_ip: std::net::IpAddr = server.get_bridge_ip_address().await.expect("bridge ip addr");
        let server_addr: String = format!("/ip4/{}/udp/4001/quic-v1", server_ip);

        containers.push(server);

        let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./client", "--dial", &bootstrap_addr])
            .with_network(network::A)
            .start()
            .await
            .expect("successful container launch");

        let client_grpc_port: u16 = client.get_host_port_ipv4(8080).await.expect("host port ipv4");
        let client_gprc_endpoint: String = format!("http://127.0.0.1:{}", client_grpc_port);
        let mut client_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(client_gprc_endpoint).await.expect("successful grpc client");

        containers.push(client);

        for _ in 0..=8 {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;

            let client_request: proto::DialRequest = proto::DialRequest {
                addr: server_addr.to_owned()    
            };

            client_grpc.dial(client_request).await.expect("successful dial");
        }

        tokio::time::sleep(std::time::Duration::from_mins(10)).await;

        // ... clean up ...
        network_a.release().await;
        network_b.release().await;

        // ... proof ...
        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");
        
        docker.write_logs_to_file(&log_dir, containers).await.unwrap();
        
        let report: log::Report = log::Report::from_dir(&log_dir).unwrap();
        assert!(report.is_proof_of_startup());
        assert!(report.is_proof_of_cohesion());
        assert!(report.is_proof_of_stability(36));
        assert!(report.is_proof_of_connectivity_persistence(6));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn end_to_end() {
    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults().unwrap();
    let mut harness: Harness = Harness::new(docker);
    harness.add_test(NatEasy);
    harness.add_test(NatHard);
    harness.add_test(Discovery);
    harness.add_test(Simulation);
    harness.launch().await;
}

async fn wait_for_grpc(endpoint: String) -> proto::node_client::NodeClient<tonic::transport::Channel> {
    loop {
        match proto::node_client::NodeClient::connect(endpoint.clone()).await {
            Ok(client) => return client,
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(500)).await,
        }
    }
}