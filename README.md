# Alternet
A decentralized network that offers a censorship-resistant alternative to traditional DNS infrastructure.

### Prerequisite
- Rust (latest stable)
- Cargo

Verify installation:

```shell
rustc --version
cargo --version
```

Viewing .excalidraw files within the repository

Suggested Obsidian
```

```

## Image

```shell
docker run --rm node:latest ./bootstrap
docker run --rm node:latest ./client
docker run --rm node:latest ./server
docker run --rm node:latest ./relay
```


## Devops

### Task
Use `task` to run custom devops scripts.

```shell
cargo run --package task
cargo run --package task build-node-release
cargo run --package task build-node
```

### Node Binaries
The `node` crate contains 4 binaries which are gated by 4 mutually exclusive features. To compile them individually, use these flags.

```shell
cargo build --release --package node --bin bootstrap --features=bootstrap
cargo build --release --package node --bin client --features=client
cargo build --release --package node --bin server --features=server
cargo build --release --package node --bin relay --features=relay
```

## License
This project is released under The Unlicense.

You are free to use, modify, distribute, and sell this software without restriction.