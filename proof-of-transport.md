
Current content is still a very rough draft.

# Requirement
- Protocol.
- It operates similar to HTTP DNS.
- Mutually assured destruction - we don't harm the regular web but attacking us harms the regular web.
- Deep packet inspection - resembles WebRTC traffic - right now we are using QUIC, but we need to mask it as WebRTC???
- realistically Uncensorable on a network level ***
- ??Blockchain
- Meet principles.
- bare-bones Node.
- Register a domain.
- Deliver any kind of data.
- DNS.
# Alternet
## Abstract
Anything can be reached.

Current transport mechanisms attempt to evade censorship through an arms-race with nation state and corporate level threats. These systems are fragile, and hurt the experience of users. ie TOR.

We propose ATL, a Proof of Transit network that is transport agnostic, to allow anyone to establish a connection with anything. It functions like a DNS, and users can use domains to point at and to anything. Unlike the modern internet, this includes IOT, resources etc

The solution is to decouple how things get there, from what gets there. Allowing diversity in methods through economic means  developers and clients don't need to care how their packets get there. If this can be cryptographic-ally proven that a packet has got to its destination, then both consumer and producer don't need to worry about the underlying infrastructure.

The ecosystem would be self adapting. The protocol would favor the best way to get the data there in terms of speed, efficiency. The diversity of relays and transport mechanisms would make it very difficult to identify the specific network, since the ecosystem evolves organically, even during high censorship environments, a system like this would incentive participants to structure their relays and paths in a way that makes them competitive.
## Context
The modern internet faces significant challenges in terms of censorship, centralized control, and reliability, particularly under adversarial conditions such as nation-state interference or large-scale infrastructure failure. Existing solutions, including TOR, VPNs, and blockchain name services like ENS, provide partial remedies but fail to offer a fully trust-less, transport-agnostic, and uncensorable network layer. The ATL, proposes a paradigm shift in network design. It separates content from transport, allowing any node or client to reach any resource, regardless of underlying network protocols or partial infrastructure outages, while ensuring verifiable delivery through POT mechanisms.
## Problem Statement
Traditional overlay networks attempt to circumvent censorship and improve privacy but suffer from fundamental limitations. TOR, for instance, relies on semi-centralized directory authorities and bandwidth-limited bridges, which are susceptible to traffic fingerprinting, resource congestion, and state-level blocking. VPNs, though encrypted and often low-latency, require trust in centralized operators and are easy to detect through deep packet inspection. ENS and similar blockchain-based naming systems provide immutable ownership and verification but are limited to mapping names to addresses or static content, without providing dynamic delivery guarantees or transport flexibility. Consequently, there exists a critical need for a network that is decentralized, trust-less, dynamically adaptive, and transport-agnostic, capable of delivering data securely and reliably even under extreme adversarial conditions.
## Design Goals
The ATL is designed with several explicit objectives:
### Trustless Delivery
Any relay or intermediary is untrusted. Proofs of packet delivery are cryptographically verifiable, ensuring relays cannot claim rewards without performing their duties.
### Transport
#### Agnosticism
The protocol decouples the method of delivery from the content, enabling relays to forward packets via any available transport (e.g., QUIC, WebRTC, TCP, Bluetooth, satellite links, or air-gapped mesh networks), mitigating the risk of protocol-specific censorship.
It shouldn't matter how it gets there, just that it does.
Most of the work is to build around this market place and capture and incentive the emergence of relays.
#### Diversity
We also want several options. This is good for a healthy system, several transport methods makes it harder to block the network.
### Multipath Redundancy
To maximize reliability and minimize censorship exposure, the same session can traverse multiple independent relay paths.
### Dynamic Adaptation
To maximize reliability and minimize censorship exposure, the same session can traverse multiple independent relay paths.
### Cryptographic Proofs
Both content integrity and delivery are verifiable via hash chains, Merkle roots, and session-level commitments, with potential extensions for zero-knowledge proofs to guarantee forwarding without revealing payload contents.
### Decentralized Governance
Protocol evolution, relay incentives, and slashing policies are administered through an onchain DAO, eliminating reliance on central authorities.
## Network Architecture
### Roles And Responsibilities
#### Client
Initiate data requests, verify relay-forwarded packets using cryptographic proofs, and manage multipath selection.
#### Server
Owners of domains, who attest onchain and advertise their location on the KAD.
#### Relay
Act as stateless or semi-stateful forwarding nodes, aggregating cryptographically signed packet chains to prove delivery and earn rewards.
#### Bootstrap Relay
Provide initial connectivity, enabling discovery of other relays, particularly in environments with partial network shutdowns.
#### Gateway Relay
Bridge ATL with deterministic systems such as blockchains, facilitating the submission of relay proofs and the redemption of incentives.


Comparison with TOR:
1. TOR still has centralized vulnerabilities from the organisation itself.
2. We are fully autonomous and onchain governed, there is no single point of failure. Payments and rewards are all crypto which bipasses traditional means of censorship.
3. TOR is slower due to limited bridge bandwidth.
4. We use multi-path, transport-agnostic routing lets relays compete to forward packets via the fastest available path. Dynamic selection could reduce latency.
5. TOR has limited capacity and congestion.
6. We have incentived relays in a competitive marketplace encourage more participants and more bandwidth. More relays will produce more throughput. Using a difficulty mechanism.
7. TOR bridges are still blockable.
8. We decouple transport. This means traffic can use QUIC, WebRTC, TCP, Bluetooth, satellite, LAN, or even air-gapped mesh methods. Blocking one transport doesnt take the network down.
9. TOR has a distinctive traffic fingerprint.
10. The protocol can mimic WebRTC or QUIC, plus dynamically change transports, making censorship detection must harder than static obfs4 bridges.
11. TOR reliance on third-party infrastructure.
12. We don't have any resilience on third party infrastructure, we are completely decentralized and transport agnostic, so it avoids central point like Cloudflare or Azure. Multiple relay paths reduce resilience on any single provider.
13. TOR has high resource usage on client devices.
14. We: potentially mitigated by letting relays handle the heavy lifting. Clients can connect over the most efficient transport available.
15. TOR's availability in heavily censored countries is questionable.
16. We: Bootstrap relays, on-chain relay registries, and multi-source discovery allow clients to find relays even if some are blocked. Offline relay discovery (USB, QR codes, LAN, Bluetooth) adds resilience.

Comparison with ENS:
1. ENS maps human-readable names to blockchain addresses or content hashes.
2. We map human-readable or structured names to any reachable network resource - servers, IOT devices, smart contracts, content, etc.
3. ENS primarily ethereum ecosystem, supports crypto payments, NFTs, content hashes.
4. We entire trustless transit network can point to any resource, onchain, or offchain, including IOT, LAN nodes, or decentralized storage.
5. ENS is onchain, relies on ethereum smart contracts.
6. We are onchain for ideetity and proofs, but dynamic offchain relay paths are allowed, fully transport-agnostic.
7. ENS: ETH addresses, contract addresses, IPFS / Arweave hashes, content metadata.
8. WE: Anything: blockchain nodes, smart contracts, IOT devices, files, server endpoints, even transport-agnostic packet destinations.
9. ENS: Can be updated via smart contract, but changes are onchain and slower.
10. WE: Can update destination dynamically, network can forward via multiple relays and transports without changing the domain name itself.
11. ENS: None, just resolves names to a fixed address/hash.
12. WE: Full, can point to TCP, QUIC, WebRTC, Bluetooth, LAN, satellite, etc.
13. ENS: Onchain, resistant to centralized DNS censorship, but resolution still requires ethereum node access.
14. We: Onchain for identity mapping, but the network layer is dynamic, multipath, and hard to block; can survive partial internet shutdowns.
15. ENS: cannot bypass firewalls, it only resolves name.
16. We: Domains can be reached through multiple transport paths, making them uncensorable in practice if relays exist.
17. ENS: Onchain ownership proof -- whoever owns the ENS record controls the name.
18. We: onchin ownership plus cryptographic keys; proof of transit ensures the domain actually reaches its intended endpoint.
19. ENS: Relies on ethereum smart contracts (public verifiable).
20. We: combines onchain verification with trustless forwarding proofs from relays from relays (signed packet chains).
21. ENS: payable addressses, nfts, decentrlized websites, smart contract refrencees.
22. We: Uncenroable messaging, IOT device addressing, decentralized CDNs, server/node reachability, multipath transport, VPN replacements

Comparison with VPN:
1. VPN: Tunnel all your internet traffic through a trusted server or servers.
2. We: send packets through a decentralized, transport-agnostic network of relays with proof of transit.
3. VPN: Usually centralized (commercial VPN) or semi-centralized (corporate VPN).
4. We: fully decentralized, relays compete to forward packets.
5. VPN: You must trust the VPN operator not to log, spy or tamper.
6. WE: Trustless, relays are untrusted, but their forwarding is cryptographically verified.
7. VPN: Ip is hidden from destination sites.
8. We: multiple hops, multipath routing, dynamic transports obscure your origin.
9. VPN: Encrypted between you and VPN server.
10. We: Can be encrypted end to end.
11. VPN: server can see traffic metadata, could be subpoenaed.
12. We: relays cannot tamper or spy without detection. signed packet chains enforce accountability.
13. VPN: moderate - timing and volume may leak.
14. We: multipath, transport diversity, and dynamic routing make traffic analysis harder.
15. VPN: works if vpn isnt blocked
16. We: works more reliably - dynamic transports, multipath routing, offline relay discovery.
17. VPN: ip addresses and traffic patterns can be blacklisted.
18. We: harder, network can reroute packets over quic, webrtc, bluetooth, satellite, LAN, etc.
19. VPN: vpns are easy to detect if DPI is used.
20. We: transport-agnostic, multihop, decoupled routing prevents single-point protocol blocks.
21. vpn: usually faster than tor, bridges single relay.
22. us: depends on relay competition, transport selection, could be faster than tor but slower than a direct vpn for a single path.
23. vpn: low latency.
24. we: moderate to high, but multipath redundancy improves reliability.
25. vpn: limited by vpn server capacity.
26. us: potentially unlimited - more relays increase throughput, incentiviced by crypto rewards.
27. vpn: no offline segments or bridging.
28. us: possible, relays can forward packets over usb, lan, bluetooth, even air-gapped links
29. vpn: no redundant paths
30. us: same packet can be sent through multiple relays to ensure delivery.
31. vpn: cannot survive partial internet shutdown.
32. us: dynamic transport, relay mesh, offline bootstrap methods.
33. vpn: high risk of detection & blocking.
34. us: multiple transports, traffic mimicking, dynamic routing.
35. vpn: full access to all traffic.
36. us: partial - relays cannot forge proofs; end to end encryption prevents content leaks.
37. vpn: single trusted server.
38. us: economic incentives + proof of transit reduce collusion.

# Risk
## Nation-State Risk
## Bootstrap Attack
## Sybil Attack
## Economic Warfare
## Governance Capture
## Full Internet Shutdown
# Governance
A DAO will govern the network.
1. Upgrade policies.
2. Incentive allocation.
3. Relay disputes and slashing.
## Upgrade Policy
# Incentive mechanism and Economics
We need to incentive a flourishing relay mesh network.
Traffic is inverse to difficulty.
When more traffic is available, difficulty drops to incentive relays to start contributing. The latter is true, as difficulty starts climbing, relays receive less rewards.
# Identity
Every participant needs a `PublicKey` and `SecretKey`. `Client` also needs one even if they do not use it, to verify their identity to the protocol both onchain and offchain.
# Domain
Domains need to support global locations, ie eth.hello-world
Every possible location, including other smart contracts have a name for them, and a packet can be sent there.

```
eth.hello-world
solana.usdc

// personal domains, you can create as many subdomains
0x04fge.fridge
0x04fg4.kitchen

// naked
node-4829
```
# Transport Agnosticism
The network does not assume any particular protocol for moving packets. Relays can use TCP, QUIC, WebRTC, Bluetooth, satellite, LAN, or any future protocol without the client or server needs to know.
## Transport Diversity

## Dynamism
The protocol itself doesn't hold ownership of the mesh network, so technologies are allowed to change and evolve to offer the best experience possible. When real world problems or obstacles take place, relays can adopt different strategies. None of this is written in code, which allows for creative behavior.
## Cryptography

```rust
type Bytes;
type Signature = Bytes;



struct Proof<T> {
	signature: Signature,
	payload: T
}


```

### Attestation


### Stack
#### Digest
Is IoT friendly and very fast.
`blake3`
#### Pair Hierarchy
##### Onchain Identity
`pqcrypto-sphincs`
Global, long-lived.
1. For onchain ownership and domain registration.
2. For signing ephemeral handshake keys for authentication.
3. For verifying integrity of content and node identity.
##### Offchain Identity
`dilithium3`
Offchain, short-to-medium-lived.
Onchain identity attests to this identity when starting up a Relay or Server.
1. For authenticating clients or ephemeral services not registered onchain.
2. For bootstrap mesh connections.
3. For rotating frequently to limit exposure.
##### Mesh Net Handshake
`Kyber-768`
Ephemeral per session.
Only used for handshake, then discarded, not enough time to break. Still PQ resistant, and competitive with regular internet.
1. For establishing session symmetric key securely.
2. For providing forward secrecy.
3. For signing identity key to prove ownership of domain.
##### Symmetric Session Key
`chacha20poly1305`
Session only, very short-lived.
1. For encrypting all packets in the session.
2. For fast, low-overhead, suitable for IoT.
3. For end-to-end encryption over multi-hop relays.



4. `pqcrypto-kyber` - Kyber - fastest pq kem, smallish keys, good for handshake
5. `pqcrypto-frodo` - FrodoKEM - conservative, lattice-based large keys
6. `pqcrypto-dilithium` - Dilithium - lattice-besed, PQ replacement for ed25519.
7. `pqcrypto-falcon` - Falcon - compact, lattice-based
8. `pqcrypto-sphincs` - SPHINCS+ - stateless, extremely securee, huge sig (~40kb)


for symmetric encryption: `chacha20poly1305` - ChaCha20-Poly1305.


### Ops
#### Registration
1. A `PublicKey` and `SecretKey` are both required onchain.
2. The `Server` can register a domain with some amount of the unit.

ONLY IF:
1. The domain has not been taken.
2. There is sufficient balance, a domain is minted in the name of the `Pk owner`.

```rust
struct Session {
	// list of offchain relays with their individual pk and sk pairs that are part of the owner's session. When relays are looking to deliver, they search for some signature that only one of those attestations could have signed. Assumes this is a valid delivery target.
	pub attestations: Vec<Attestation>
}

struct Record {
	pub owner: OnchainPublicKey,
	
	// option because the owner may want to signal that there is no reachable session for this domain yet or at the moment.
	pub session: Option<Session>,
	
	// the unix timestamp that the ownership of this domain will expire on, can be renewed, but after this time it will become available for someone else to mint again.
	pub expiry: u64
}
```

#### Session Launch
A new relay joins the network.
1. Must generate a pk and sk offchain using dilithium3.
2. Update the record onchain with an attestation that this pair is a valid delivery target.

how do relays find this server?? based on records onchain?

3. On the network, this node needs to bootstrap and connect to the network.

```rust
struct SessionSignal {
	pub domain: String,
	pub session_public_key: PublicKey,
	pub multiaddrs: Vec<Multiaddr>,
	pub timestamp: u64
}

Signed<SessionSignal>;
```

#### Dialing
- Relays need to be incentived to move data.
- Using POT it should guarantee that relays pick up tasks.

- How to guarantee transport diversity...



1. Client must first produce cryptographic proof that they 
2. Client looks up domain onchain (now has offchain pk of recipient).
3. Client adds recipient to first `Packet` which contains `Handshake`.
4. Handshake contains the client's public key and is signed by the Client.

```rust
struct Handshake {
	pub signer: PublicKey,
	pub recipient_public_key: PublicKey
}
```

4. H 

|     |                                                                        |
| --- | ---------------------------------------------------------------------- |
| 0   | Client looks up domain onchain (now has offchain pk of the recipient). |
| 1   | Client add recipient to first `Packet` which contains handshake.       |
|     |                                                                        |


Dial through `Multiaddr` on Kad. on Kad the record will contain a signature with matching the canonical truth onchain, this is checked by delays to find the right record.

The record is returned to the relay looking for it which is the one in contact with the client. The relay checks this and should start dialing the 

Even in a case where the relay attempts to point to a wrong server or host, this problem will be resolved through the handshake

### Packet Level Encryption
- Speed is important.
- Use handshake for the session - use asymmetric encryption to share symmetric encryption secret key between both parties for faster interaction. Session Handshake.

Protocol for handshake.
1. Identity
2. Signing
3. Encryption
4. Commitments
5. Proof of Transit

# Topology
## Relay
- Forwarding.
- Proof aggregation.
- Handling NAT / firewalls.
## Client
- Path selection.
- Redundant delivery.
- Verifying packets from server through relay.


# The Four Pillars
```
ring = "*" // for crypto
libp2p??
```

## Kernel

```rust
type Signature;

struct Signed<T> {
	signature: Signature,
	content: T
}

// onchain this checks that the signer has x available to mint the domain, if the domain is available.
struct Registration {
	pub signer: String,
	pub domain: String
}

trait Kernel {
	fn register(Signed<Registration>);
}
```

### Partial ERC20 - Domain Unit

The domain units are partially compliant with current ERC20 standards.

```protobuf
message Transfer {
	uint32 version = 1;
	bytes signature = 2;
	bytes sender = 3;
	bytes recipient = 4;
	bytes amount = 5;
	uint64 nonce = 6;
	uint64 ttl = 7;
}
```

```rust
trait Erc20 {
	type Address;
	type Balance;
	
	fn total_supply() -> Self::Balance;
	fn balance_of(owner: Self::Address) -> Self::Balance;
	
	fn transfer(
		from: Self::Address,
		to: Self::Address,
		amount: Self::Balance
	) -> Result<()>;
	
	
}
```

### Architecture

What blockchain to use??
Hopefully one that supports Rust as its safer than Solidity.

ERC2535 Diamond Proxy for upgradeability.
Onchain source of truth. Holds proof of ownership, domains, and uses onchain pk and sk pairs to proof.
The ENS record contains??:
1. Owner key
2. Content hash (IPFS CID, Arweave hash)
3. Node ID or multi-address (optional, could be static)
4. Public key for verifying content

> Immutable, verifiable, canonical reference.

### Relay Registry

## Aggregators

## Server or Host
Expose multiple `Multiaddr` for the host, for transport diversity. To give relays multiple options on how to reach it.

SERVER COLLUSION??
If a server receives on first, but choose to sign a later one?

what if lock a certain session, not per packet. lock in 1 hour session, relays must guarantee that path for that hour - client may only use up $0.01 to make it cheap?

The place to be reached, can be a phone, watch, IOT device, or another chain, etc.
Serves content, forwards messages, handles discovery.
1. Participates in the network directly.
2. Stores, generates, or consumes data.
3. Direct connections.
4. Can store state, cache, or full content.
5. Advertise itself, may be discoverable.
6. Serve some content and communicate with clients.

Needs to handle:
7. P2P overlay and discovery.
8. Content-addressed storage.
9. Transport (QUIC/WebRTC mimic).
10. Messaging / PubSub (gossipsub or floodsub).
11. Peer management / NAT traveral (hole punching, relay support).
#### Hook
Once the kernel returns the hash to reach the server, the server has an address on the dht of there to find them?? what is it distance dependent? what if they are far? we can proove that server put it there from eth pk and network pk, but this process is foggy
### Relay - Mesh Network?? Overlay Network?? Routing??
Anyone can write a relay, it just needs to proove it forwarded it to the destination. In fact, transport doesnt matter much, they just need to get it to point A to Point B, and maybe back.

Each relay along the path gets rewarded...

#### Decoupled Transport 
Many networks assume a single protocol (HTTP, TCP, WebSocket, etc.). If that protocol is blocked, the network becomes unreachable.

By decoupling transport, relays are free to forward data however they like:
- QUIC, WebRTC, TCP, Bluetooth, LAN, satellite links... anything that can carry the payload.
- The relay's choice is independent of the client's transport.

This allows the network tot adapt dynamically if one method is blocked.
##### Advantages
The network is transport-agnostic.
###### No Single Point Of Protocol Censorship
Blocking HTTP or TCP only affects some relays, not all.
###### Multi-path delivery
Clients can receive the same data via multiple relays using different transports, making it hard to block.
###### Redundant paths
Relays can choose paths that avoid surveillance or national firewalls.
###### Dynamic fallback
If a relay can't reach the destination over one transport, it can try another automatically.
###### Offline Resilience
Relays can even use USB, LAN, or Bluetooth to bridge air-gapped segments.



The relays have some api on them, they must conform. One of those is for searching for the path

Facilitates connection to the chain, nodes, and client.
1. Forwards traffic between nodes that cannot connect directly.
2. No direct connections.
3. Only facilitates indirect connectivity.
4. Stateless and ephemeral.
5. Just passes packets/messages.
6. Often hidden.
7. Used for reachability.
8. A TURN server in WebRTC, or Nostr relay.

Need to handle re connection if it cant find another relay. If no one else, try to reach back to the sender with this problem.

Needs to handle:
1. Minimal p2p forwarding.
2. TURN-like relay for WebRTC.
3. Stateless routing / ephemeral.
If you want **network-level uncensorable mesh**, **libp2p relay** is easiest and fully Rust-native.
If you want **browser-friendly relay**, combine libp2p with WebRTC transport or use `webrtc-rs`.
#### Handled By `libp2p`
1. Peer discovery.
2. Multi-hop relay.
3. Address advertisement.
#### Handled By Us
Forwarding protocol that signs `PacketChain`.
#### Regular Relay


##### Custom
We want transport diversity, so we want to allow TCP, QUIC, WebRTC, Bluetooth, etc.. This guarantees that in any case of severe censorship, there is at least a relay with a means to reach the required destination.


#### Bootstrap Relay
These relays should be gateways and know a lot of other relays, should also handle navigation ??
How to avoid censorship at this layer.
1. WebRTC  HTTPS or CDN to communicate with bootstrap relay, after that communication with other relays may optionally use any transport as long as it gets to the destination or just require WebRTC etc for all?? Blocking this means blocking Cloudflare, GoogleCloud, etc.

What about onchain bootstrap??

Multisource Discovery:
1. Random DNS seeds??
2. ENS names
3. IPFS-hosted relay lists
4. DHT-based discovery

Full internet shutdown!
1. Users manually share relay multiaddrs
2. QR codes
3. USB transfers
4. LAN discovery
5. Bluetooth mesh
#### Gateway Relay
Facilitates connectivity to blockchains which are deterministic environments, they cannot reach out into the work. So the gateway reads state of a gateway contract onchain which packets to ship, once shipped, returns the proof of delivery to the chain. The same works the other way around, it pushes this onchain. Due to extra expenses, we may need additional incentives. Possible expansion into zktls for allowing onchain infrastructure to reach out into the internet.
### Client
- Client generates `SessionKey` `S123`.
- Client selects path: `RelayA` -> `RelayB` -> `Server`.
- Client creates `PathCommitment(S123, [RelayA, RelayB])`.
- Client signs commitment -> sends to RelayA.
- RelayA signs -> forwards to RelayB.
- RelayB signs -> forwards to Server.

```rust
type SequenceIndex;
type SessionKey;
type Bytes;


struct Signed<T> {
	Signature,
	T
}

struct Packet {
	pub sequenceIndex: u64,
	pub content: Bytes
}

// signer is the sender
// the
type SignedPacket = Signed<Packet>;



struct PathCommitmenet {

}

struct MultiPathCommitment(SessionKey, Vec<PathCommitment>);



```

#### Native CLI
#### Browser
Browser level extension to facilitate communication with the network with help for NAT and looking for a relay or node directly.

Connect via WebRTC transport to some relay.

Initially connects to a publicly reachable `BootstrapRelay`.
## Mesh
1: Client requst to mesh network 
2: Request forwarded to chain 
3: Chain responds with signature and relevant data of the place to go to mesh.
4: Mesh forwards back to client
5: Client reaches out to mesh with the node it needs to visit
6: Mesh forwards to node.
7: Node proves it is the correct one by signing the message.
8: forward to mesh
9: Client has proof the node holds the sk,
10: Initiate socket or bidirectional stable connection to peer (mesh network now needs to guarantee this path for the length of the session, if a relay closes, a new path must bee guaraanteed??)
### Relay Incentive
The relays are untrusted, they need to prove they have successfully send things through.
1. Proove you have relayed the clients signed transaction to the chain.
2. Proove you have sent back the response to the client
3. Proove you sent the handshake hello to the node/server.
4. Proove you sent the handshake hello from the server to the client.
5. Proove you have send back and forth packets of data to and from the server.
Relays can then redeem some reward.
#### Session-Level Signed Acknowledgements
1. Client signs each message it receives (or a summary of them).
2. Node signs each response.
3. Relay aggregates these signed messages into a `Batch` or `Merkle` root.
4. Submit the `Batch` or `Root` to the contract.
##### Enhancements
1. Sequence numbers - no replay attacks or packet omission.
2. Timestamps - prove timely delivery.
3. Multipath redundancy - multiple relays can all produce independent proofs, settle rewards per path.
4. Off-chain aggregation - keep detailed proofs off-chain and only submit summary roots to reduce gas.
##### Future Expansion
Zero-knowledge / Verifiable Forwarding
- Relays generate ZKSNARK/ZKSTARK proof that they forwarded a session of messages without revealing content.
- Can be verified **trustlessly on-chain**
- Fully independent of client/server honesty
- Much heavier to implement, probably overkill for initial prototype
Constraint:
- Relay is untrusted
- Client and Node may refuse to confirm
- Proof should be verifiable on-chain
- Should not reveal the actual content of packets

Hash chain??
Merkle root...

```



# Component
## `Receiver`
We encourage nodes to listen on multiple addresses, we need a way to make this easy for them. Instead of parsing tcp or udp, etc differently we can offer a common abstraction that receives ordered data.

`libp2p` solves this for us.

```rust

```

## `PathFinder`
Present on both Server and Client. 

`PathReservation`
A cryptographic commitment where all nodes along the paths must sign that the packet must and has been forwarded by them.

`PersistentPathReservation`

Session and Path Binding
# Proof Of Transport POT
# Constructs
## `Packet`
Unit of transport.

- needs to support multi-hop forwarding.
- signed packet chains
- path commitments
- replay protection.
- encryption
- transport-agnosticism
- proof-of-transit aggregation

```rust
// every dock or relay the cargo stopped at.
struct Stop {
	pub relay_public_key: PublicKey,
	pub arrival_hash: Hash,
	pub departure_hash: Hash,
	pub timestamp: u64
}

// a packet
// 1. signing for others will dilute reward
// 2. hoping from relay to relay will reduce reward of ttl
// 3. failure to deliver cargo will yield no rewards
// 4. tampering with it will yield no rewards even if delivered
// 5. delivering the cargo late will yield no rewards
// 6. the client must be able to proove they have the funds to reward this onchain (done cryptographically)
// 7. the recipient is the one who signs and returns a final proof that this packet as successfully made it to its destination.
// 8. no guarantee it will be delivered, so ttl allows the client to know that if it wasnt nothing can be delivered.
// 9. trying to sniff through its contents will delay packet delivery and nullify rewards or reduce them.
// 10. if server refuses to sign and grant the relays work, relays can refuse to work with them, reputation is crucial for both sides.
// 11. relay reputation based on rewards aquired, server and client rep based on rewards sent, and serve on confirmations. all parties are incentivized to act well as reputation is storedd onchain at the highest level of truth. starting a new account means your cargo may be picked up from risk takers, so having positive rep, means relays will race to pick up your sessions
struct Packet {
	pub signer: PublicKey,
	pub session_key: u64, // session commitment
	pub session_sequence_index: u64, // avoids replay
	pub data: Vec<u8>,
	pub ttl: u64,
	pub session_root: Hash,
	pub dst: Vec<Signed<Stop>> 
}

Signed<Cargo> - signed by client
```

## `PacketChain`

```rust
struct PacketChain {
	pub session_key: Vec<u8>,
	pub packets: Vec<Packet>,
	pub chain_hashes: Vec<Vec<u8>>
}
```

## `MerkleAggregation`
```rust
struct MerkleAggregation {
	pub session_key: Vec<u8>,
	pub merkle_root: Vec<u8>,
	pub packet_chains: Vec<PacketChain>
}
```

## `SessionCommitment`
```rust
struct SessionCommitment {
    pub session_key: Vec<u8>,
    pub commitment_root: Vec<u8>,
    pub timestamp: u64
}
```

## `Receipt`

The receipt issued by either a `Server` or `Client` for the `PacketChain` received. These are handed down to each `Relay` responsible for getting the `Packet` to its destination. To avoid middle `Relay`s being omitted, the client or the sender locks a path and generates a `PathCommitment`.

```rust
struct Receipt {
	pub session_key: Vec<u8>,
	pub packet_chain_root: Vec<u8>,
	pub path_hash: Vec<u8>,
	pub final_sequence: u64,
	pub signer: Vec<u8>,
	pub signature: Vec<u8>
}
```

## Milestone
### 0.x.x Proof Of Concept (POC)
- This may take 1 - 2 months.
- We already have some work done which we can use for to pivot for this plan.
- This will contain no user interface.
- Will at most have a functioning CLI.
- Will not contain any blockchain specific logic - no guarantee.
- We will build traits and some sort of interface to be able to plug blockchain interaction later.
- Will not be production ready.
- Needs to include 4 libraries:
	- Server SDK.
	- Relay SDK.
	- Bootstrap Relay SDK.
	- Client SDK.
- Needs to include 4 binaries:
	- Sever - showcasing how to use the SDK to make a functioning system.
	- Relay - showcasing how to make a minimum viable Relay.
	- Bootstrap Relay - showcasing a minimum viable Bootstrap Relay.
	- A CLI client - showing how to interact with the network.
- Test - A test to prove that given a situation where a bootstrap node is available, a client.
#### 0.1.x Single Relay PoT
Client sends packet through relay to server. Verifies hash chain. Establishes basic forwarding and verification.
##### Feature
1. Client sends a packet to a server via one relay.
2. Relay forwards packets and signs hash chain for proof-of-transit.
3. Server receives packets, signs hash chain back to relay.
4. CLI shows end-to-end verification of packet delivery.
#### 0.2.x Multi-Hop PoT (2 - 3 relays)
Multi-hop forwarding (2–3 relays). Fallback if relay fails.
##### Feature
1. Multi-hop relay forwarding.
2. Hash chain extended across hops.
3. CLI shows verification per hop.
#### 0.3.x Multi-path forwarding
Redundant paths for reliability. Same session delivered via multiple paths.
##### Feature
1. Client sends packets over 2+ paths.
2. Relays forwards independently.
3. CLI shows multi-path verification.
#### 0.4.x Transport-Agnosticism
Dynamic transport selection (QUIC/WebRTC/TCP). Network adapts if a transport fails.
##### Feature
1. Relays can forward via multiple transports.
2. Client selects transports dynamically.
3. CLI shows which transport is used per hop.
#### 0.5.x Proof Aggregation & Merkle Chains
Aggregate proofs into merkle root. Session-level PoT works across multiple packets & paths.
##### Feature
1. Relays aggregate signed packet chains.
2. Client and server verify merkle root.
3. CLI displays session-level summary.
#### 0.6.x Bootstrap & End-to-End
Full POC with bootstrap, relay discovery, multi-path, transport diversity, proof aggregation. Complete trust-less forwarding demo ready to showcase.
##### Feature
1. Bootstrap relay discovery.
2. Multi-hop & multi-path forwarding.
3. Dynamic transports.
4. Aggregated session proof.
5. CLI shows full session (bootstrap, relay mesh, server, return).
### 1.x.x TBC