#![cfg(feature = "end-to-end")]

use std::io::Read as _;
use futures_util::StreamExt as _;
use futures_util::TryStreamExt as _;
use testcontainers::ImageExt as _;
use testcontainers::runners::AsyncRunner as _;

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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn end_to_end() {
    std::process::Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("task")
        .arg("build-image")
        .spawn()
        .expect("")
        .wait()
        .expect("");

    let workspace_dir: std::path::PathBuf = cargo_metadata::MetadataCommand::new()
        .exec()
        .unwrap()
        .workspace_root
        .to_string()
        .into();

    let image_dir: std::path::PathBuf = workspace_dir
        .join("target")
        .join("image");

    let bootstrap_image_path: std::path::PathBuf = image_dir.join("bootstrap.tar");
    
    println!("{:?}", bootstrap_image_path);

    let client_image_path: std::path::PathBuf = image_dir.join("client.tar");
    let server_image_path: std::path::PathBuf = image_dir.join("server.tar");
    let relay_image_path: std::path::PathBuf = image_dir.join("relay.tar");

    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults().unwrap();
    docker.load(&bootstrap_image_path).await;
    docker.load(&client_image_path).await;
    docker.load(&server_image_path).await;
    docker.load(&relay_image_path).await;

    let container_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
    
    let container_images: Vec<_> = vec![
        testcontainers::GenericImage::new("relay", "latest")
            .with_exposed_port(container_port)
            .with_env_var("PUBLIC_KEY", "x")
            .with_env_var("SECRET_KEY", "y")
            .start()
            .await
            .unwrap(),
        testcontainers::GenericImage::new("server", "latest")
            .with_exposed_port(container_port)
            .with_env_var("PUBLIC_KEY", "x")
            .with_env_var("SECRET_KEY", "y")
            .start()
            .await
            .unwrap(),
        testcontainers::GenericImage::new("client", "latest")
            .with_exposed_port(container_port)
            .with_env_var("PUBLIC_KEY", "x")
            .with_env_var("SECRET_KEY", "y")
            .start()
            .await
            .unwrap(),
        testcontainers::GenericImage::new("server", "latest")
            .with_exposed_port(container_port)
            .with_env_var("PUBLIC_KEY", "x")
            .with_env_var("SECRET_KEY", "y")
            .start()
            .await
            .unwrap()
    ];
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
}