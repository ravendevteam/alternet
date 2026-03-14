This crate builds to 3 independent binaries `Bootstrap`, `Client`, `Server`, `Relay`. Only one of these features can be enabled at a time.

You should have `protoc` installed needed to build gRPC proto.


Where the standard variants are designed to be maximally beneficial to the network, their malicious counterparts are used for testing and are designed to be maximally malicious.

The degree of resilience is determined by how many malicious nodes the network can handle, the aim is to see this grow.

Malicious behaviour should not reflect actions or behaviour that act against the best interest of the actor, only tests are done for realistic attempts.


## System Overview


## Node Archetypes
The system uses Rust feature flags to compile specific node roles. Standalone binaries are extremely lightweight, and role specific.

| Role      | Responsability |
|-----------|----------------------------|
| Bootstrap | Network entry point and DHT seed. |
| Relay     | Facilitates NAT traversal and traffic forwarding.
| Server    | Domain operator node; provides services.
| Client    | End-user node, resolves and consumes services.

> Malicious variants are compiled with additional sub-systems to validate network resilience under stress. Where normal nodes are designed to be maximally good to the network, malicious variants are designed to be maximally bad. As the project evolves, the network will handle more and more aggresive behaviours from malicious variants.



3. Networking Stack
The POC leverages the libp2p modular stack to handle the complexities of peer-to-peer communication.

3.1 Transport & Security
QUIC (Primary): Selected for its native support for stream multiplexing, reduced handshake latency, and superior performance in lossy network environments.

Noise & Yamux: Used as fallback security and multiplexing layers for non-QUIC or relayed connections.


3.2 Discovery & Routing (Kademlia)The network uses the Kademlia DHT for distributed peer discovery.Routing Table: Nodes maintain a k-bucket-based routing table. The $k$ value is typically 20, though our POC scales this based on node role (e.g., Bootstrap nodes maintain larger caches of 50,000 identities).Query Parallelism ($\alpha$): Set to 32 for Bootstrap nodes to ensure high availability during network formation.


3.3 NAT Traversal (DCutr)
To solve the "restrictive NAT" problem, the system implements Direct Connection Upgrade through Relay (DCutr).

Reservation: A Client/Server node makes a reservation on a Relay.

Observation: Nodes use the Identify protocol to learn their external multiaddresses.

Hole Punching: When two nodes behind NATs wish to connect, they use the Relay as a signaling channel to synchronize a synchronized UDP hole-punching attempt via QUIC.

## Control Plane (gRPC)
Each node exposes a programmatic interface via `gRPC` (using `tonic`). This allows CLI tooling to:
1. Trigger manual dials.
2. Inspect routing table health.
3. Validate cryptographic proofs of service availability.