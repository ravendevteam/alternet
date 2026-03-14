

# 0.1.x
## Network Topology
The simulation environment utilizes Docker based network isolation to mimic real-world NAT constraints.



### Routing Logic
Nodes in Subnet A cannot reach Subnet B directly without relay assistance.


## Test Methodology

1. Orchestration - spin up a bootstrap node, followed by relay, server, and client nodes.
2. Discovery - monitor kad logs to ensure dht population.
3. Hole punching - trigger a connection between a client and server through the relay node using QUIC-first transport.
4. Adversarial injection - introduce `malicious_relay` nodes configured to attempt routing table pollution.
5. Log analysis - parse structured node logs to verify cryptographic identity binding and routing table stability.



## Observed Behaviour & Results

| Criteria             | Result | Evidence                                                                      |
|----------------------|--------|-------------------------------------------------------------------------------|
| Node Compilation     | PASS   |  |
| Peer Discovery       | PASS   | KAD logs confirm `bootstrap complete, remaining: 0`.                          |
| NAT Traversal        | PASS   | Successful hole punching confirmed across subnet A/B boundaries.              |
| Routing Stability    | PASS   | `is_proof_of_stability` check confirms churn dropped below threshold.         |
| Sybil Resistance     | PASS   | Honest peers remained reachable despite malicious routing pollution.          |
| Identity Persistence | PASS   | `is_proof_of_unique_identity` verified no collisions in ephemeral IDs.        |

### Adversarial Condition Analysis

During the Sybil simulation, `malicious_relay`s attempted to saturate the routing table with invalid peer entries. The system maintained 100% functionality, while lookup latency increased, honest nodes remained in the routing table, and connectivity was never fully severed.




## Observation
### Simulation





```bash
cargo run --package task build-image
cargo test --features=end_to_end

```



# End to End test hygene
Make sure to release the network before assetions.

# Reproduction Instructions
cargo test --features end_to_end

Docker must be installed and running.

The test harness automatically builds the node container image and executes the simulation suite.

# Network Topology


# Environment Assumption
- Docker container networking
- Simulated NAT via segmented docker networks.
- QUIC transport over UDP.
- libp2p Kademlia routing.

# Test Methodology
Nodes launched inside isolated docker networks.

Tests simulate:

1. permissive NAT
2. restrictive NAT
3. multi-peer discovery
4. adversarial routing pollution

Validation is performed through structured log analysis.

# CLI
The nodes expose grpc, you can online grpc to connect to it.


# Architectural Documentation

