# (0.x.x) Proof Of Concept
A decentralized DNS and networking layer that enables client to reach servers through domain-based lookup, with a primary focus on anti-censorship rather than privacy or anonymity. The system is designed to interoperate with the existing web stack to reduce fingerprint-ability and increase resilience against blocking. It will leverage QUIC (with potential WebRTC support), incorporate KAD for network-level domain discovery, and including NAT traversal mechanisms.

The POC introduces a modular abstraction layer for blockchain integration through adapters and traits, enabling domain resolution and economic logic to interface with external on-chain systems. Central to this design is the incorporation of cryptographic proofs that bind network behavior, such as domain ownership assertions, service attestations, relay participation, and transport verification to economic mechanisms. These proofs form the foundational primitive for incentive alignment and reward distribution within the network.

To support deterministic testing and validation prior to integration with live blockchain environments, the milestone includes the implementation of a mock chain and associated test assets. This simulated on-chain environment models domain registration, ownership state transitions, proof verification, and reward distribution, thereby enabling the network to function as a fully self-contained system under reproducible conditions. The architecture is designed such that subsequent integration with production on-chain systems can occur with minimal modification to the core protocol and networking layers.

The POC delivers at least 3 primary binaries:
1. Client: A lightweight node responsible for domain resolution, routing, NAT traversal, on-chain state validation, and verification of cryptographic proofs associated with domain and incentive logic.
2. Server: A node intended for domain operators, incorporating on-chain interaction, service attestation logic, reward pool participation, and generation of cryptographic proofs relating to domain control and service availability.
3. Relay: A lightweight intermediary node that facilitates connectivity and routing, optionally generating or verifying transport-level cryptographic proofs and participating in incentive-aligned relay selection mechanisms.

CLI tooling will be provided as separate binaries layered atop node APIs. Depending on architectural finalization, this may be discrete CLI binaries per node type or a unified CLI interface with subcommands. These tools interface with nodes via exposed programmatic endpoints (RPC, JSON-RPC, gRPC, or related protocols).

The deliverable is CLI-only (no graphical interface) and not production-ready. It includes unit and end-to-end testing (approximately 50% coverage) and reproducible simulation environments designed to validate protocol correctness and network behavior. The milestone further includes formalized network design documentation and architectural analysis serving as a precursor to a comprehensive technical  specification, as well as repository structuring and build stabilization to support subsequent research and development phases.
### Validation & Documentation Standard (Applies to All Milestones)
*For every milestone*

All validation criteria listed within each milestone are considered incomplete unless accompanied by corresponding documentation describing the test conditions, methodology, and observed behavior. Execution alone does not constitute milestone completion; reproducibility and documentation are required deliverables.

Each milestone includes structured documentation that clearly defines:
- Network topology and node roles.
- Environmental assumptions (configuration, blocking models, adversarial behavior).
- Test methodology and execution flow.
- Definitions of successful behavior.
- Observed behavior under both normal and adversarial conditions.
- Reproduction steps (configuration, scripts, environment setup).
### Completion Standard
A milestone is considered complete when:
- All listed validation criteria are implemented and demonstrated.
- All validation criteria are documented according to the Validation & Documentation Standard.
- Associated artifacts are delivered in repository form with reproducible instructions.
### Scope & Quality Boundaries
This project is explicitly defined as a research-stage Proof of Concept (POC).

The validation documentation required under each milestone is intended to support reproducibility and clarity of results, not to serve as end-user documentation or comprehensive production specifications.

Future production-readiness efforts would require separate scoping, budgeting, and security review.
### Research-Stage Validation Interpretation
Because this project is a research-stage POC, validation criteria are intended to evaluate architectural viability and observable system behavior rather than guarantee production-grade performance under all possible conditions.

Where limitations, instability, or partial failures are encountered:
- The behavior must be reproducible.
- The cause must be documented.
- The architectural implications must be analyzed.

A milestone is considered complete when the validation criteria have been meaningfully implemented and evaluated, and resulting behavior, including limitations, is documented according to the Validation & Documentation Standard.

Unanticipated architectural constraints or environmental inconsistencies discovered during validation may result in design adjustments within the scope of the milestone without being considered non-completion.
## (0.1.x) Foundation - $2,000
### Research Objective
Validate that decentralized connectivity can function under NAT conditions with QUIC-first design and KAD discovery.
### Validation Criteria
- All node binaries compile and launch independently in a clean environment.
- CLI successfully connects to and interacts with node endpoints.
- Peer discovery operates correctly within a controlled multi-node simulation including honest and adversarial peers.
- Hole punching succeeds across multiple distinct network configurations representing materially different routing behaviors (permissive vs restrictive).
- Routing tables stabilize after initial network formation and maintain connectivity despite adversarial routing attempts.
- Ephemeral peer identifiers are used during simulations and are shown to limit trivial peer enumeration.
- A Sybil scenario is simulated in which a subset of peers attempt routing table pollution.
- Under adversarial conditions, honest peers remain present in routing tables, lookup functionality remains operational, though performance degradation may be observed.
### Artifact
- Functional client, server, relay binaries.
- CLI tool(s).
- Structured workspace and repository administration is cleanly maintained.
- Initial architectural documentation.
- KAD integration.
- NAT traversal implementation.
- Multi-node test harness.
- Foundation milestone validation report documenting network topology, attack simulation, network configuration, and measured outcomes.
## (0.2.x) Mock Chain Implementation - $500
### Research Objective
Validate that economic logic and proof submission can operate in mock environment before real-chain integration.
### Hypothesis
A simulated onchain environment can fully model domain ownership, lookup, proof validation, and reward distribution without modifying code protocol logic.
### Validation Criteria
- Domain registration and ownership transitions execute within the mock chain environment.
- Proof submission updates simulated economic state in a deterministic manner.
- Replay attempts and invalid proofs are rejected according to defined rules.
- Cached and gossiped domain state remains consistent with mock chain state.
- Determinism assumptions and replay protections are documented.
- Reproducibility of state transitions and validation behavior is demonstrated and documented.
### Artifact
- Mock chain executable.
- Onchain trait interface.
- CLI interaction tooling.
- Deterministic simulation environment.
- Mock chain validation documentation describing state modeling, determinism guarantees, and proof handling assumptions.
## (0.3.x) Adaptive Network Resilience - $4,000
### Research Objective
Validate that the protocol can maintain connectivity under common network-layer censorship techniques through adaptive transport selection, relay redundancy, and dynamic peer indirection.

This milestone focuses on measurable resilience against IP blocking, port filtering, basic protocol filtering, and connection disruption without requiring full protocol obfuscation or anonymity guarantees.
#### Threat Model
The protocol is evaluated against a network-level adversary capable of:
- IP-based blocking.
- Port-based filtering.
- Basic protocol fingerprinting.
- Simple DPI heuristics.
- Active connection resets.
- Relay IP enumeration.
- Bootstrapping targeting.
- Selective peer hiding and ephemeral relay rotation.

The milestone does not assume advanced nation-state adversaries performing large-scale traffic correlation or full protocol mimicry detection.
### Hypothesis
An adaptive connectivity strategy combining:
- Multi-transport dialing.
- Parallel direct and relay attempts.
- Dynamic relay discovery and rotation.
- Domain-based peer indirection.

will measurably increase successful connection rates under simulated filtering conditions compared to a static single-transport design.
#### Implementation Scope
The following mechanisms will be implemented:
##### Transport Agility
- Direct QUIC dialing.
- QUIC over relay.
- TCP fallback (where supported).
- Automatic escalation on failure.
##### Parallel Dial Strategy
- Concurrent direct and relay attempts.
- First-success path selection.
- Automatic downgrade when direct path is blocked.
##### Multi-Relay Architecture
- Dynamic relay selection.
- Relay capability advertisement via KAD.
- Circuit rotation across sessions.
##### Domain-Based Indirection
- Domain resolution via KAD (resolution implies being able to locate the a record left by the owner of a domain).
- Domain maps to rotating peer identities.
- Peer identities map to dynamic relay reservations or direct addresses.
##### Failure Escalation Logic
- Detect connection resets.
- Retry with alternate relay.
- Retry with alternate transport.
- Maintain bounded retry strategy.
##### Bootstrap Redundancy
- Multiple bootstrap peers.
- No single static bootstrap dependency.
### Validation Criteria
Resilience is evaluated using a deterministic blocking simulation harness capable of emulating filtering and connection disruption scenarios.

Success is defined by:
- Observable improvement under at least one filtered condition compared to static direct-only configuration.
- Automatic recovery behavior under connection disruption.
- Consistent failover behavior across repeated simulation runs.
- Observable relay rotation when blocking conditions are introduced.

The following are recorded and reported:
- Connection outcomes under baseline and filtered conditions.
- Transport path selection behavior.
- Fail-over sequence and recovery patterns.
- Relay rotation behavior.

All blocking assumptions, measurement methodology, and observed behavior are documented in a resilience evaluation report.
### Artifact
- Blocking and filtering simulation harness.
- Instrumented connection telemetry (transport selection and failover logs).
- Relay discovery and scoring implementation.
- Escalation strategy implementation.
- Measurement report.
- Technical documentation describing adaptive connectivity design and trade-offs.
## (0.4.x) Proof System - $2,000
### Research Objective
Validate that cryptographic proofs can bind network behavior to economic outcomes.
### Hypothesis
Network-level events such as domain control, service availability, relay participation can be cryptographically proven in a way that is verifiable, replay-resistant, economically enforceable.
### Validation Criteria
- Valid proofs are accepted by the verification system.
- Invalid or replayed proofs are rejected.
- Expired proofs are invalidated.
- Proof lifecycle and replay protections are documented.
- Economic state transitions resulting from proofs are observable and reproducible in simulation.
### Artifact
- Proof schema definitions.
- Verification engine.
- Mock chain integration.
- Proof lifecycle documentation.
## (0.5.x) End-to-End Composition - $1,500
### Research Objective
Validate full protocol flow under reproducible simulation.
### Hypothesis
All system components can operate coherently:
- Resolution
- Connectivity
- Proof Generation
- Economic Settlement
### Validation Criteria
- Domain resolution succeeds in multi-node simulation.
- Server responds to client requests.
- Relay fallback engages when direct connection fails.
- Proof submission and economic updates occur as expected.
- Failure scenarios are observable and documented.
- End-to-end interaction flow and failure modes documented.
### Artifact
- Multi-node reproducible test environment.
- End-to-end scenario documentation.
- Failure mode analysis.
- End-to-end validation report.