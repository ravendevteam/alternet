#![allow(clippy::enum_variant_names)]

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(clap::Parser)]
struct Main {
    #[command(subcommand)]
    command: Command
}

#[derive(clap::Subcommand)]
enum Command {
    BuildImage,
    
    #[command(name = "build-node")]
    BuildNode,

    #[command(name = "build-node-release")]
    BuildNodeRelease,
    NodeGrpcDescribe {
        #[arg(long)]
        port: usize
    },
    Ping {
        #[arg(long)]
        port: usize,
        #[arg(long)]
        message: String
    }
}

trait DockerExt {
    async fn import_image_tar(&self, path: &std::path::Path) -> Result<()>;
    async fn export_image_to_tar_from_ws_context(
        &self, 
        ws_root: &std::path::Path, 
        ws_root_exclude: &[std::path::PathBuf],
        image_name: &str,
        image_tag: &str,
        image_out_dir: &std::path::Path,
        dockerfile: &str,
    ) -> Result<()>;
}

impl DockerExt for bollard::Docker {
    async fn import_image_tar(&self, path: &std::path::Path) -> Result<()> {
        use std::io::Read as _;
        use futures_util::TryStreamExt as _;
        let mut file = std::fs::File::open(path)?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        let conf: bollard::query_parameters::ImportImageOptions = bollard::query_parameters::ImportImageOptionsBuilder::new()
            .quiet(false)
            .build();
        let mut stream = self.import_image(conf, bollard::body_full(buf.into()), None);
        while let Some(_) = stream.try_next().await? {

        }
        Ok(())
    }

    async fn export_image_to_tar_from_ws_context(
        &self, 
        ws_root: &std::path::Path, 
        ws_root_exclude: &[std::path::PathBuf],
        image_name: &str,
        image_tag: &str,
        image_out_dir: &std::path::Path,
        dockerfile: &str,
    ) -> Result<()> {
        use std::io::Write as _;
        use futures_util::TryStreamExt as _;
        std::fs::create_dir_all(image_out_dir)?;
        let image_full_name: String = format!("{}:{}", image_name, image_tag);
        let image_out_path: std::path::PathBuf = image_out_dir.join(format!("{}.tar", image_name));
        let mut tar: tar::Builder<_> = tar::Builder::new(vec![]);
        let dockerfile_bytes: &[_] = dockerfile.as_bytes();
        let dockerfile_bytes_len: u64 = dockerfile_bytes.len() as u64;
        let mut dockerfile_header: tar::Header = tar::Header::new_gnu();
        dockerfile_header.set_path("Dockerfile")?;
        dockerfile_header.set_size(dockerfile_bytes_len);
        dockerfile_header.set_mode(420);
        dockerfile_header.set_cksum();
        tar.append(&dockerfile_header, dockerfile_bytes)?;
        for item in walkdir::WalkDir::new(ws_root)
            .into_iter()
            .filter_entry(|item| {
                let rel: &std::path::Path = item
                    .path()
                    .strip_prefix(ws_root)
                    .unwrap_or(item.path());
                let rel_str: std::borrow::Cow<_> = rel.to_string_lossy();
                !ws_root_exclude.iter().any(|path| rel_str.starts_with(path.to_str().unwrap()))
            }) {
            let item: walkdir::DirEntry = item?;
            let path: &std::path::Path = item.path();
            let rel_path: &std::path::Path = path.strip_prefix(ws_root)?;
            if rel_path.as_os_str().is_empty() {
                continue
            }
            if path.is_file() {
                tar.append_path_with_name(path, rel_path)?;
            } else if path.is_dir() {
                tar.append_dir(rel_path, path)?;
            }
        }
        tar.finish()?;
        let buf: Vec<_> = tar.into_inner()?;
        let body = bollard::body_full(buf.into());
        let conf: bollard::query_parameters::BuildImageOptions = bollard::query_parameters::BuildImageOptionsBuilder::new()
            .dockerfile("Dockerfile")
            .t(&image_full_name)
            .rm(true)
            .pull("true")
            .build();
        let mut build_stream = self.build_image(conf, None, Some(body));
        while let Some(msg) = build_stream.try_next().await? {
            if let Some(s) = msg.stream {
                print!("{}", s);
            }
            if let Some(error) = msg.error_detail {
                let error: Box<_> = format!("docker build failed: {:?}", error).into();
                return Err(error)
            }
        }
        let mut export_stream = self.export_image(&image_full_name);
        let mut file: std::fs::File = std::fs::File::create(&image_out_path)?;
        while let Some(chunk) = export_stream.try_next().await? {
            file.write_all(&chunk)?;
        }
        file.flush()?;
        Ok(())
    }
}

fn workspace_dir() -> Result<std::path::PathBuf> {
    let output: std::process::Output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .output()?;
    if !output.status.success() {
        let error: Box<dyn std::error::Error> = "".into();
        return Err(error)
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let dir: std::path::PathBuf = metadata["workspace_root"]
        .as_str()
        .ok_or("missing `workspace_root`")?
        .into();
    Ok(dir)
}

#[tokio::main]
async fn main() -> Result<()> {
    use clap::Parser as _;
    let main: Main = Main::parse();
    match &main.command {
        Command::BuildImage => {
            let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults()?;
            let ws_root: std::path::PathBuf = workspace_dir()?;
            let ws_root_exclude: Vec<&str> = vec![
                "target"
            ];
            let ws_root_exclude: Vec<std::path::PathBuf> = ws_root_exclude
                .iter()
                .map(|s| s.into())
                .collect();
            let image_name: &str = "node";
            let image_tag: &str = "latest";
            let image_out_dir: std::path::PathBuf = ws_root
                .join("target")
                .join("image");
            let dockerfile: String = r#"
                FROM rust:1.93.1-slim as builder
                WORKDIR /app
                COPY . .
                ENV RUSTFLAGS="-Awarnings"
                RUN apt-get update
                RUN apt-get install -y protobuf-compiler
                RUN rm -rf /var/lib/apt/lists/*
                RUN cargo build --release --package node --bin bootstrap --features=bootstrap --no-default-features
                RUN cargo build --release --package node --bin client --features=client --no-default-features
                RUN cargo build --release --package node --bin server --features=server --no-default-features
                RUN cargo build --release --package node --bin relay --features=relay --no-default-features
                RUN cargo build --release --package node --bin malicious_bootstrap --features=malicious_bootstrap --no-default-features
                RUN cargo build --release --package node --bin malicious_client --features=malicious_client --no-default-features
                RUN cargo build --release --package node --bin malicious_server --features=malicious_server --no-default-features
                RUN cargo build --release --package node --bin malicious_relay --features=malicious_relay --no-default-features
                FROM debian:bookworm-slim
                WORKDIR /app
                COPY --from=builder /app/target/release/bootstrap .
                COPY --from=builder /app/target/release/client .
                COPY --from=builder /app/target/release/server .
                COPY --from=builder /app/target/release/relay .
                COPY --from=builder /app/target/release/malicious_bootstrap .
                COPY --from=builder /app/target/release/malicious_client .
                COPY --from=builder /app/target/release/malicious_server .
                COPY --from=builder /app/target/release/malicious_relay .
            "#
            .to_owned();
            docker.export_image_to_tar_from_ws_context(&ws_root, &ws_root_exclude, image_name, image_tag, &image_out_dir, &dockerfile).await?;
        },
        Command::BuildNode => {
            let roles: [_; _] = [
                "bootstrap",
                "client",
                "server",
                "relay"
            ];
            for role in roles {
                std::process::Command::new("cargo")
                    .arg("build")
                    .arg("--package")
                    .arg("node")
                    .arg("--bin")
                    .arg(role)
                    .arg(format!("--features={}", role))
                    .arg("--no-default-features")
                    .spawn()?
                    .wait()?;
            }
        },
        Command::BuildNodeRelease => {
            let roles: [_; _] = [
                "bootstrap",
                "client",
                "server",
                "relay"
            ];
            for role in roles {
                std::process::Command::new("cargo")
                    .arg("build")
                    .arg("--release")
                    .arg("--package")
                    .arg("node")
                    .arg("--bin")
                    .arg(role)
                    .arg(format!("--features={}", role))
                    .arg("--no-default-features")
                    .spawn()?
                    .wait()?;
            }
        },
        Command::NodeGrpcDescribe {
            port
        } => {
            std::process::Command::new("grpcurl")
                .arg("--import-path")
                .arg("./app/node/proto/")
                .arg("--proto")
                .arg("an.proto")
                .arg("-plaintext")
                .arg(format!("0.0.0.0:{}", port))
                .arg("describe")
                .arg("an.Node")
                .spawn()?
                .wait()?;
        },
        Command::Ping {
            port,
            message
        } => {
            std::process::Command::new("grpcurl")
                .arg("--import-path")
                .arg("./app/node/proto")
                .arg("--proto")
                .arg("an.proto")
                .arg("-plaintext")
                .arg("-d")
                .arg(format!(r#"'{{"msg": "{}"}}'"#, message))
                .arg(format!("0.0.0.0:{}", port))
                .arg("an.Node/Ping")
                .spawn()?
                .wait()?;
        }
    }
    Ok(())
}