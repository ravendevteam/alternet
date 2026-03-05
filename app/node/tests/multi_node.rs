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

static IMAGE_LOADED: tokio::sync::OnceCell<std::sync::Arc<tokio::sync::Mutex<bool>>> = tokio::sync::OnceCell::const_new();

async fn load_image(docker: &bollard::Docker) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = IMAGE_LOADED.get_or_init(async || {
        std::sync::Arc::new(tokio::sync::Mutex::new(false))
    });
    let loaded: &std::sync::Arc<_> = loaded.await;
    let mut loaded: tokio::sync::MutexGuard<_> = loaded.lock().await;
    if *loaded {
        return Ok(())
    }
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
    docker.load(&image_path).await;
    *loaded = true;
    Ok(())
}

async fn reset_containers(docker: &bollard::Docker) -> Result<(), Box<dyn std::error::Error>> {
    let containers: Vec<_> = docker.list_containers(None).await?;
    for container in containers {
        docker.stop_container(&container.id.to_owned().unwrap(), None).await?;
    }
    Ok(())
}

async fn reset_network(docker: &bollard::Docker, network_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let containers: Vec<_> = docker.list_containers(None).await.unwrap();
    for container in containers {
        let Some(id) = container.id else {
            continue
        };
        let request: bollard::secret::NetworkDisconnectRequest = bollard::secret::NetworkDisconnectRequest {
            container: id,
            force: Some(true)
        };
        docker.disconnect_network(network_name, request).await.ok();
    }
    Ok(())
}

trait DockerEngine {
    async fn load(&self, path: &std::path::Path);
}

impl DockerEngine for bollard::Docker {
    async fn load(&self, path: &std::path::Path) {
        
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
    }
}

async fn write_logs_to_file(docker: &bollard::Docker, container_id: &str, path: &std::path::Path) {
    let logs_conf: bollard::query_parameters::LogsOptions = bollard::query_parameters::LogsOptions {
        stdout: true,
        stderr: true,
        timestamps: true,
        tail: "all".into(),
        ..Default::default()
    };
    let mut file: tokio::fs::File = tokio::fs::File::create(path).await.unwrap();
    let mut stream = docker.logs(container_id, Some(logs_conf));
    while let Some(log) = stream.next().await {
        let log: bollard::container::LogOutput = log.unwrap();
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



struct VirtualNetwork {
    docker: bollard::Docker,
    containers: Vec<testcontainers::ContainerAsync<testcontainers::GenericImage>>
}

impl VirtualNetwork {
    pub async fn new(docker: bollard::Docker) -> Result<Self, Box<dyn std::error::Error>> {
        if false {
            std::process::Command::new("cargo")
                .arg("run")
                .arg("--package")
                .arg("task")
                .arg("build-image")
                .spawn()
                .expect("")
                .wait()
                .expect("");
        }
        
        let workspace_dir: std::path::PathBuf = cargo_metadata::MetadataCommand::new()
            .exec()
            .unwrap()
            .workspace_root
            .to_string()
            .into();
        
        let image_dir: std::path::PathBuf = workspace_dir
            .join("target")
            .join("image");

        let image_path: std::path::PathBuf = image_dir.join("node.tar");

        docker.load(&image_path).await;

        let containers: Vec<_> = vec![];
        let new: Self = Self {
            docker,
            containers
        };
        Ok(new)
    }
}

impl VirtualNetwork {
    pub async fn launch_bootstrap_node(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let container_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let container: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(container_port)
            .with_env_var("PUBLIC_KEY", "x")
            .with_env_var("SECRET_KEY", "y")
            .with_cmd(["./bootstrap"])
            .with_network("an")
            .start().await?;
        self.containers.push(container);
        Ok(())
    }

    pub async fn launch_client_node(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let container_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let container: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(container_port)
            .with_env_var("PUBLIC_KEY", "x")
            .with_env_var("SECRET_KEY", "y")
            .with_cmd(["./client"])
            .with_network("an")
            .start().await?;
        self.containers.push(container);
        Ok(())
    }

    pub async fn launch_server_node(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let container_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let container: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(container_port)
            .with_env_var("PUBLIC_KEY", "x")
            .with_env_var("SECRET_KEY", "y")
            .with_cmd(["./server"])
            .with_network("an")
            .start()
            .await?;
        self.containers.push(container);
        Ok(())
    }

    pub async fn launch_relay_node(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let container_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let container: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(container_port)
            .with_env_var("PUBLIC_KEY", "x")
            .with_env_var("SECRET_KEY", "y")
            .with_cmd(["./relay"])
            .with_network("an")
            .start()
            .await?;
        self.containers.push(container);
        Ok(())
    }

    pub async fn write_logs_to_dir(&self, log_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        tokio::fs::create_dir_all(log_dir).await?;

        for container in &self.containers {
            let id = container.id();
            let path = log_dir.join(format!("{}.log", id));

            write_logs_to_file(&self.docker, &id, &path).await;
        }

        Ok(())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn e2e() {
    // End to end tests must be executed sequentially because they make use of docker and a lot of external tools, this keeps the tests
    // predictable, otherwise, the tests may be flaky.

    // end_to_end().await;
    nat_easy().await;
    nat_hard().await;
}

async fn end_to_end() {
    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults().unwrap();

    // load_image(&docker).await.unwrap();

    let log_dir: std::path::PathBuf = std::path::PathBuf::new()
        .join("tests")
        .join("log")
        .join("end_to_end");

    let network: &str = "an";
    let network_udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
    let network_tpc_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

    reset_containers(&docker).await.ok();
    reset_network(&docker, network).await.ok();

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

    tokio::fs::remove_dir_all(&log_dir).await.expect("unable to cleanup logs");
    tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

    for container in containers {
        let container_id: &str = container.id();
        let container_path: std::path::PathBuf = log_dir.join(format!("{}.log", container_id));

        write_logs_to_file(&docker, container_id, &container_path).await;
    }
}

async fn nat_easy() {
    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults().unwrap();

    // load_image(&docker).await.unwrap();

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

    reset_containers(&docker).await.ok();
    reset_network(&docker, network_a).await.ok();
    reset_network(&docker, network_b).await.ok();

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

    for log in logged {
        let log_id: &str = log.id();
        let log_path: std::path::PathBuf = log_dir.join(format!("{}.log", log_id));

        write_logs_to_file(&docker, log_id, &log_path).await;
    }

    docker.remove_network(network_a).await.ok();
    docker.remove_network(network_b).await.ok();
}

async fn nat_hard() {
    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults().unwrap();

    // load_image(&docker).await.unwrap();

    let log_dir: std::path::PathBuf = std::path::PathBuf::new()
        .join("tests")
        .join("log")
        .join("nat_easy");

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

    reset_containers(&docker).await.ok();
    reset_network(&docker, network_a).await.ok();
    reset_network(&docker, network_b).await.ok();

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

    for log in logged {
        let log_id: &str = log.id();
        let log_path: std::path::PathBuf = log_dir.join(format!("{}.log", log_id));

        write_logs_to_file(&docker, log_id, &log_path).await;
    }

    docker.remove_network(network_a).await.ok();
    docker.remove_network(network_b).await.ok();
}