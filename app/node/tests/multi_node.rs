#![cfg(feature = "end-to-end")]

use std::io::Read as _;
use futures_util::StreamExt as _;
use futures_util::TryStreamExt as _;
use testcontainers::ImageExt as _;
use testcontainers::runners::AsyncRunner as _;
use tokio::io::AsyncWriteExt;

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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn end_to_end() {
    std::process::Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("task")
        .arg("build-image")
        .spawn()
        .expect("failed to build image")
        .wait()
        .expect("failed to build image");

    let log_dir: std::path::PathBuf = std::path::PathBuf::new()
        .join("tests")
        .join("log");
    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults().unwrap();
    let network: &str = "an";
    let network_udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
    
    let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
        .with_exposed_port(network_udp_port)
        .with_cmd(["./bootstrap"])
        .with_network(network)
        .start()
        .await
        .unwrap();
    let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.unwrap();
    let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

    let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
        .with_exposed_port(network_udp_port)
        .with_cmd(["./relay", "--dial", &bootstrap_addr])
        .with_network(network)
        .start()
        .await
        .unwrap();

    let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
        .with_exposed_port(network_udp_port)
        .with_cmd(["./server", "--dial", &bootstrap_addr])
        .with_network(network)
        .start()
        .await
        .unwrap();

    let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
        .with_exposed_port(network_udp_port)
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

    tokio::time::sleep(std::time::Duration::from_mins(3)).await;
    
    tokio::fs::remove_dir_all(&log_dir).await.expect("unable to cleanup logs");
    tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

    for container in containers {
        let container_id: &str = container.id();
        let container_path: std::path::PathBuf = log_dir.join(format!("{}.log", container_id));

        write_logs_to_file(&docker, container_id, &container_path).await;
    }
}