#![allow(clippy::vec_init_then_push)]
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


struct Log {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: String,
    pub component: String,
    pub message: String
}

impl std::str::FromStr for Log {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (timestamp, more) = s.split_once(' ').ok_or("")?;
        let timestamp: chrono::DateTime<chrono::Utc> = timestamp.parse()?;
        let lb: usize = more.find('[').ok_or("")?;
        let rb: usize = more.find(']').ok_or("")?;
        let meta: &str = &more[lb + 1..rb];
        let (severity, component) = meta.split_once(' ').ok_or("")?;
        let message: String = more[lb + 1..].trim().to_owned();
        let severity: String = severity.to_owned();
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

struct Logs(std::collections::HashMap<String, Vec<Log>>);

impl Logs {
    pub fn from_file(paths: &[&std::path::Path]) -> Result<Self> {
        let mut new: std::collections::HashMap<String, Vec<Log>> = std::collections::HashMap::new();
        for path in paths {
            let file_name: std::ffi::OsString = path.file_name().ok_or("")?.to_owned();
            let file_name: String = file_name
                .to_string_lossy()
                .to_string();
            let (file_name, extension) = file_name.split_once('.').ok_or("")?;
            if extension != "log" {
                let error: Box<dyn std::error::Error> = "".into();
                return Err(error)
            }
            let file_name: String = file_name.to_owned();
            let content: String = std::fs::read_to_string(path)?;
            let mut logs: Vec<Log> = vec![];
            for line in content.lines() {
                let log: Log = line.parse()?;
                logs.push(log);
            }
            new.insert(file_name, logs);
        }
        Ok(Self(new))
    }
}

impl Logs {
    pub fn proof_of_eventual_discovery(&self) {

    }

    pub fn proof_of_successful_bootstrap(&self) {
        let complete: bool = self.0.iter().any(|line| {
            line.message.contains("bootstrap complete, remaining: 0")
        });
        assert!(complete, "bootstrap never reached 0 remaining nodes");
    }

    pub fn proof_of_causality(
        &self,
        lhs_container_id: &str,
        lhs_log_message: &str,
        rhs_container_id: &str,
        rhs_log_message: &str
    ) {
        let lhs_time: chrono::DateTime<_> = self.0[lhs_container_id].iter()
            .find(|line| {
                line.message.contains(lhs_log_message)
            })
            .map(|line| {
                line.timestamp
            })
            .expect("lhs event never happend");
        let rhs_time = self.0[rhs_container_id].iter()
            .find(|line| {
                line.message.contains(rhs_log_message)
            })
            .map(|line| {
                line.timestamp
            })
            .expect("rhs event never happend");
        assert!(lhs_time < rhs_time, "causality violation: rhs event happend before lhs event");
    }

    pub fn proof_of_convergence(&self, expected_peer_count: usize, limit: std::time::Duration) {
        for (id, logs) in self.0.iter() {
            let reached = logs.iter().any(|line| {
                line.message.contains("knows about") && line.message.contains(&format!("{} peers", expected_peer_count))
            });
            assert!(reached, "container {} never converged to {} peers", id, expected_peer_count);
        }
    }

    pub fn proof_of_convergence_within_duration() {

    }

    pub fn proof_of_connectivity_persistence(&self, min_peers: usize) {
        for (id, logs) in &self.0 {
            let mut converged: bool = false;
            for line in logs {
                if line.message.contains("knows about") && line.message.contains(&format!("{} peers", min_peers)) {
                    converged = true;
                }
                if converged && line.message.contains("peer count dropped") {

                    panic!("container {} lost connectivity after converging", id);
                }
            }
        }
    }
}



trait DockerEngine {
    async fn load(&self, path: &std::path::Path) -> Result<()>;
    async fn load_built_tar_image_from_ws_target_dir(&self) -> Result<()>;
    async fn reset(&self) -> Result<()>;
    async fn reset_network(&self, network_name: &str) -> Result<()>;
    async fn write_logs_to_file(&self, out_dir: &std::path::Path, containers: Vec<testcontainers::ContainerAsync<testcontainers::GenericImage>>) -> Result<()>;
}

impl DockerEngine for bollard::Docker {
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
            let container_path: std::path::PathBuf = out_dir.join(container_id);
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

struct Suite {
    docker: bollard::Docker,
    tests: Vec<Box<dyn Test>>
}

impl Suite {
    pub fn new(docker: bollard::Docker) -> Self {
        let tests: Vec<_> = vec![];
        Self {
            docker,
            tests
        }
    }
}

impl Suite {
    pub fn register<T>(&mut self, test: T)
    where
        T: Test,
        T: 'static {
        self.tests.push(Box::new(test));
    }

    pub async fn run(&mut self) {
        self.docker.reset().await.ok();
        self.docker.load_built_tar_image_from_ws_target_dir().await.unwrap();
        for test in self.tests.iter() {
            test.run(&self.docker).await;
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

        let network_a: &str = "an-a";
        let network_a_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network_a.to_owned(),
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

        let network_b: &str = "an-b";
        let network_b_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network_b.to_owned(),
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

        docker.reset().await.ok();
        docker.reset_network(network_a).await.ok();
        docker.reset_network(network_b).await.ok();
        docker.create_network(network_a_conf).await.expect("successful network creation");
        docker.create_network(network_b_conf).await.expect("successful network creation");

        let udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let tcp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

        let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./bootstrap"])
            .with_network(network_a)
            .start()
            .await
            .expect("successful container launch");

        let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.expect("bridge ip addr");
        let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

        let bootstrap_network_connection_conf: bollard::secret::NetworkConnectRequest = bollard::secret::NetworkConnectRequest {
            container: bootstrap.id().to_owned(),
            endpoint_config: None
        };

        docker.connect_network(network_b, bootstrap_network_connection_conf).await.expect("successful network connection");

        let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./relay", "--dial", &bootstrap_addr])
            .with_network(network_a)
            .start()
            .await
            .expect("successful container launch");

        let relay_network_connection_conf: bollard::secret::NetworkConnectRequest = bollard::secret::NetworkConnectRequest {
            container: relay.id().to_owned(),
            endpoint_config: None
        };

        docker.connect_network(network_b, relay_network_connection_conf).await.expect("successful network connection");

        let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./server", "--dial", &bootstrap_addr])
            .with_network(network_b)
            .start()
            .await
            .expect("successful container launch");

        let server_ip: std::net::IpAddr = server.get_bridge_ip_address().await.expect("bridge ip addr");
        let server_addr: String = format!("/ip4/{}/udp/4001/quic-v1", server_ip);

        let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./client", "--dial", &bootstrap_addr])
            .with_network(network_a)
            .start()
            .await
            .expect("successful container launch");

        let client_grpc_port: u16 = client.get_host_port_ipv4(8080).await.expect("host port ipv4");
        let client_gprc_endpoint: String = format!("http://127.0.0.1:{}", client_grpc_port);
        let mut client_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(client_gprc_endpoint).await.expect("successful grpc client");

        let client_request: proto::DialRequest = proto::DialRequest {
            addr: server_addr    
        };

        client_grpc.dial(client_request).await.expect("successful dial");

        tokio::time::sleep(std::time::Duration::from_mins(1)).await;

        let mut logged: Vec<_> = vec![];
        logged.push(bootstrap);
        logged.push(relay);
        logged.push(server);
        logged.push(client);

        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

        docker.write_logs_to_file(&log_dir, logged).await.unwrap();
        docker.remove_network(network_a).await.ok();
        docker.remove_network(network_b).await.ok();
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

        let network_a: &str = "an-a";
        let network_a_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network_a.to_owned(),
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

        let network_b: &str = "an-b";
        let network_b_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network_b.to_owned(),
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

        docker.reset().await.ok();
        docker.reset_network(network_a).await.ok();
        docker.reset_network(network_b).await.ok();
        docker.create_network(network_a_conf).await.expect("successful network creation");
        docker.create_network(network_b_conf).await.expect("successful network creation");

        let udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let tcp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

        let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./bootstrap"])
            .with_network(network_a)
            .start()
            .await
            .expect("successful container launch");

        let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.expect("bridge ip addr");
        let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

        let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./relay", "--dial", &bootstrap_addr])
            .with_network(network_a)
            .start()
            .await
            .expect("successful container launch");

        let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./server", "--dial", &bootstrap_addr])
            .with_network(network_b)
            .start()
            .await
            .expect("successful container launch");

        let server_ip: std::net::IpAddr = server.get_bridge_ip_address().await.expect("bridge ip addr");
        let server_addr: String = format!("/ip4/{}/udp/4001/quic-v1", server_ip);

        let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./client", "--dial", &bootstrap_addr])
            .with_network(network_a)
            .start()
            .await
            .expect("successful container launch");

        let client_grpc_port: u16 = client.get_host_port_ipv4(8080).await.expect("host port ipv4");
        let client_gprc_endpoint: String = format!("http://127.0.0.1:{}", client_grpc_port);
        let mut client_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(client_gprc_endpoint).await.expect("successful grpc client");

        let client_request: proto::DialRequest = proto::DialRequest {
            addr: server_addr    
        };

        client_grpc.dial(client_request).await.expect("successful dial");

        tokio::time::sleep(std::time::Duration::from_mins(1)).await;

        let mut logged: Vec<_> = vec![];
        logged.push(bootstrap);
        logged.push(relay);
        logged.push(server);
        logged.push(client);

        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

        docker.write_logs_to_file(&log_dir, logged).await.unwrap();
        docker.remove_network(network_a).await.ok();
        docker.remove_network(network_b).await.ok();
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

        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

        docker.write_logs_to_file(&log_dir, containers).await.unwrap();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn e2e() {
    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults().unwrap();
    let mut suite: Suite = Suite::new(docker);
    suite.register(NatEasy);
    suite.register(NatHard);
    suite.register(Discovery);
    suite.run().await;
}