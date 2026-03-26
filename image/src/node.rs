use super::*;

pub struct Node;

#[async_trait::async_trait]
impl Image for Node {
    async fn render() -> Result<GenericImage> {
        GenericBuildableImage::new("node", "latest")
            .with_dockerfile_string(
                r#"
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
                    RUN apt-get update
                    RUN apt-get install -y iproute2
                    RUN apt-get install -y iptables
                    RUN apt-get install -y iputils-ping
                    RUN apt-get install -y ca-certificates
                    RUN apt-get install -y curl
                    RUN rm -rf /var/lib/apt/lists/*
                    COPY --from=builder /app/target/release/bootstrap .
                    COPY --from=builder /app/target/release/client .
                    COPY --from=builder /app/target/release/server .
                    COPY --from=builder /app/target/release/relay .
                    COPY --from=builder /app/target/release/malicious_bootstrap .
                    COPY --from=builder /app/target/release/malicious_client .
                    COPY --from=builder /app/target/release/malicious_server .
                    COPY --from=builder /app/target/release/malicious_relay .
                    EXPOSE 8080/tcp
                    EXPOSE 4001/udp
                "#
            )
            .with_file(ws_root()?.join("app"), "app")
            .with_file(ws_root()?.join("lib"), "lib")
            .with_file(ws_root()?.join("image"), "image")
            .with_file(ws_root()?.join("image_util"), "image_util")
            .with_file(ws_root()?.join("task"), "task")
            .with_file(ws_root()?.join("Cargo.toml"), "Cargo.toml")
            .with_file(ws_root()?.join("Cargo.lock"), "Cargo.lock")
            .build_image()
            .await
            .map_err(testcontainers::TestcontainersError::into)
    }
}


// failed to build the image 'node:latest', error: Docker stream error: process "/bin/sh -c apt-get iputils-ping" did not complete successfully: exit code: 100