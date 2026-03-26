use super::*;

pub struct Router;

#[async_trait::async_trait]
impl Image for Router {
    async fn render() -> Result<GenericImage> {
        GenericBuildableImage::new("router", "latest")
            .with_dockerfile_string(
                r#"
                    FROM rust:1.93.1-slim-bookworm as builder
                    WORKDIR /app
                    COPY . .
                    ENV RUSTFLAGS="-Awarnings"
                    RUN rm -rf /var/lib/apt/lists/*
                    RUN cargo build --release --package router
                    FROM debian:bookworm-slim
                    WORKDIR /app
                    RUN apt-get update
                    RUN apt-get install -y iproute2
                    RUN apt-get install -y iptables
                    RUN apt-get install -y ca-certificates
                    run apt-get install -y curl
                    RUN rm -rf /var/lib/apt/lists/*
                    COPY --from=builder /app/target/release/router .
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
            // "./router: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by ./router)\n",
    }
}

