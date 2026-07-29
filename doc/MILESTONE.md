# (0.x.x) Proof Of Concept
A research-stage Proof of Concept validating the architectural feasibility of a decentralized domain resolution and networking layer with anti-censorship objectives. The system evaluates whether clients can reach servers through domain-based lookup using QUIC-first transport, KAD-based discovery, NAT traversal, relay-assisted connectivity, and adaptive transport strategies under controlled simulation conditions.

The design includes a modular abstraction layer for blockchain integration, allowing domain ownership state, cryptographic proof verification, and economic transitions to operate within a deterministic mock chain environment. Cryptographic proofs bind observable network behavior, such as domain control, service availability, and relay participation, to simulated economic state changes.

The POC delivers three primary binaries: client, server, and relay nodes, along with CLI tooling layered over exposed node APIs. It includes a mock chain executable, multi-node simulation harness, adversarial testing scenarios, and structured validation documentation to support reproducible evaluation of network behavior and architectural coherence.

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

## 0.3 Adaptive Network Resilience

### 0.3.1 Transport $800
#### Criteria
- Establish primary node-to-node transport connections, and initial stream handling utilizing QUIC, and TLS as core supported protocols.
- Implement multi-transport mechanism to enable automatic switching between transports.
- Configure fully reproducible transport build and testing environments.
- Demonstrate successful primary transport connection establishment using QUIC, and TLS.
- Verify basic transport fallback behaviour between supported connection configurations.
- Confirm reproducible build, and test automation setup for transport components.

### 0.3.2  Connection Failure Resilience $800
#### Criteria
- Enhance node-level stability during active network disruptions.
- Detect connection drops, timeouts, and resets automatically.
- Trigger escalation routines to recover alternate paths without loosing operational state.
- Demonstrate automated detection of broken or dropping connections.
- Verify automatic path recovery under simulated network failure.

### 0.3.3 Dynamic Selection $800
#### Criteria
- Discover available intermediary routing nodes dynamically.
- Evaluate intermediary node performance, latency, and availability.
- Route traffic dynamically through optimal intermediary candidates.
- Demonstrate dynamic lookup and evaluation of intermediary nodes.
- Verify successful indirect traffic routing through selected nodes

### 0.3.4 Concurrent Connection Handling $800
#### Criteria
- Attempt multiple connection paths simultaneously.
- Demonstrate successful path selection under concurrent dialing conditions.
- Verify resource lifecycle stability and absence of hung connection states.

### 0.3.5 Domain Indirection $800
#### Criteria
- Evaluate connectivity under simulated network filtering and disruption scenarios.
- Demonstrate end-to-end connectivity recovery during simulated blocking.
- Deliver a high-level resilience evaluation report summarizing system behavior.

## 0.4 Proof System

### 0.4.1 Proof $1000
#### Criteria
- Establish mechanisms to generate and verify cryptographic proofs for domain control and node participation.
- Enforce proof lifecycle rules, including expiration and replay prevention.
- Demonstrate acceptance of valid cryptographic proofs.
- Verify automatic rejection of expired, altered, or replayed proofs.

### 0.4.2 Mock Chain Settlement $1000
#### Criteria
- Connect verified cryptographic proofs to the simulated state environment.
- Trigger deterministic state updates and economic reward updates upon proof submission.
- Demonstrate reproducible state transitions in response to valid proof submissions.
- Verify state consistency across repeated simulation executions.

## 0.5 Integration

### 0.5.1 End-to-End System Integration $800
#### Criterial
- Connection resolution, connectivity, proof generation, and economic settlement into a unified workflow.
- Ensure client, server, relay, bootstrap, and mock chain components interact harmoniously.
- Prepare executable builds and deployment artifacts.
- Demonstrate full end-to-end execution across all node types in a unified environment.
- Provide executable builds accompanied by release instructions.

### 0.5.2 Repository Refactoring, Polish & Final Documentation
#### Criteria
- Perform broad codebase refactoring and general improvements across all modules.
- Conduct code stabilization, and final repository cleanup.
- Synthesize overall research findings into final project documentation.
- Verify final codebase quality, cleanliness, and build stability.
- Deliver the failure mode analysis report and overall Proof of Concept summary document.