Betanet Version 1.1 – Official Implementation Specification
Normative document. All requirements marked MUST, MUST NOT, or SHALL are mandatory for compliance.

0 Status & Scope
Betanet is a fully decentralised, censorship-resistant network intended to replace the public Internet. This revision finalises covert transport indistinguishability, removes linkability vectors, specifies liveness for naming, hardens governance and bootstrap economics, and standardises adaptive calibration.

1 General Encoding Rules
Multi-byte integers: unsigned big-endian.
varint: QUIC variable-length integer (RFC 9000 §16).
Unless stated, sizes are in bytes.
Binary examples use hexadecimal.
2 Cryptography
Purpose	Primitive
Hash	SHA-256 (32)
AEAD	ChaCha20-Poly1305 (IETF, 12-B nonce, 16-B tag)
KDF	HKDF-SHA256
Signatures	Ed25519
Diffie–Hellman	X25519
Post-quantum hybrid	X25519-Kyber768 (hybrid)
PQ requirement. From 2027-01-01, the inner key agreement (L2 §5.3) MUST be hybrid X25519-Kyber768. The outer TLS handshake (L2 §5.1) MUST mirror the front origin and MUST NOT advertise PQ that diverges from the origin’s canonical fingerprint.

Algorithm agility. Implementations MUST expose a registry of cryptographic parameters keyed by OIDs; future substitutions MUST be negotiated via governance (L7) without changing wire formats where lengths permit.

3 Layer Model
Layer	Function
L0	Access media (any IP bearer: fibre, 5G, sat, LoRa, etc.)
L1	Path selection & routing (SCION + HTX-tunnelled transition)
L2	Cover transport (HTX over TCP-443 / QUIC-443)
L3	Overlay mesh (libp2p-v2 object relay)
L4	Optional privacy hop (Nym mixnet)
L5	Naming & trust (self-certifying IDs + 3-chain alias ledger)
L6	Payments (federated Cashu + Lightning)
L7	Applications
4 Path Layer (L1)
4.1 SCION Packet Header
0       1       2       3
+-------+-------+-------+-------+
|Ver=0x2|Reserved|  Type        |
+-------------------------------+
|     Total Header Length       |
+-------------------------------+
|       Payload Length          |
+-------------------------------+
|      Path Segment 0 …         |
+-------------------------------+
Ver MUST be 0x02.
Type MUST be 0x01 (single path) or 0x03 (path list).
Each AS-hop signature in every segment MUST verify before forwarding; otherwise drop.
4.2 Transition Across Non-SCION Links (HTX-Tunnelled)
Gateways bridging non-SCION segments MUST encapsulate SCION packets inside an HTX control session (L2) between gateways:

Establish an HTX session using §5 (origin-mirrored TLS).

Negotiate a Transition Control Stream opened by the gateway acting as the server, with stream_id=2 (first even stream).

The control stream payload is a single CBOR map:

{ prevAS, nextAS, TS, FLOW, NONCE, SIG }
where TS = unix-sec, FLOW = 8B random, NONCE = 8B random, and SIG = Ed25519 over (prevAS ‖ nextAS ‖ TS ‖ FLOW ‖ NONCE ‖ "bn-t1").

Gateways MUST verify TS within ±300 s, reject duplicate (FLOW,TS) for 2 h, and apply per-peer token buckets.

SCION payloads are carried on subsequent HTX streams. No per-packet transition header appears on the public wire.

Gateways MUST close and re-establish the control stream on rekey (§5.3). Out-of-date control streams are invalid.

Public-Internet requirement. The legacy on-wire transition header is PROHIBITED on public networks. It MAY be used only on private, administratively controlled links (§11).

4.3 Path Maintenance
Maintain up to 3 disjoint validated paths per peer.
Switch within 300 ms when an alternative validated path exists.
Probe with exponential back-off (min 1 s, max 60 s); MUST NOT exceed 1 probe per destination per 10 s under persistent loss.
5 Cover Transport (L2) — HTX
5.1 Outer TLS 1.3 Handshake (Origin Mirroring & Auto-Calibration)
Clients MUST mirror the front origin’s fingerprint class (JA3/JA4 family, ALPN set, extension order, GREASE, H2 SETTINGS, H3 availability).
A per-connection calibration pre-flight to the same origin (or same CDN POP if indicated) MUST be performed before inner handshake; any deviation MUST fail closed.
Tolerances: ALPN set and order MUST match exactly. Extension order MUST match exactly. H2 SETTINGS values MUST be within ±15 % of calibrated origin values where the origin publishes variable ranges; otherwise MUST match exactly.
POP selection: If the origin presents geo/POP variance, clients MUST calibrate against the same POP as the target connection as indicated by authoritative DNS co-location or CDN route headers; otherwise calibration MUST be repeated until equality is achieved.
ALPN selection MUST match the origin; fixed global distributions are PROHIBITED.
Session resumption MUST follow origin policy; 0-RTT MUST NOT be used for HTX initiation.
5.2 Access-Ticket Bootstrap (Negotiated Carrier, Replay-Bound)
Decoy site publishes ticketPub (X25519, 32B), ticketKeyID (8B), and a carrier policy describing allowed carriers and probabilities:

BN-Ticket: v1; carriers=cookie:0.5,query:0.3,body:0.2; len=24..64
Client generates cliPriv/cliPub (X25519) and 32-B nonce32.

sharedSecret = X25519(cliPriv, ticketPub).

hour = floor(unixTime/3600) (UTC).

salt = SHA256("betanet-ticket-v1" ‖ ticketKeyID ‖ uint64_be(hour)).

accessTicket = HKDF(sharedSecret, salt, "", 32).

Client sends exactly one carrier chosen per policy with variable-length padding to match len:

Cookie (recommended): Cookie: =Base64URL( 0x01 ‖ cliPub32 ‖ ticketKeyID8 ‖ nonce32 ‖ accessTicket32 ‖ pad ) A __Host- prefix is RECOMMENDED.
Query parameter: ...?bn1=
Body (POST) with Content-Type: application/x-www-form-urlencoded containing bn1=.
Server verification:

recompute for hour ∈ {now-1, now, now+1},
reject duplicates for tuple (cliPub, hour) within 2 h,
enforce per-/24 IPv4 and /56 IPv6 token buckets.
Servers MUST parse fields in order (version, cliPub32, ticketKeyID8, nonce32, accessTicket32) and MUST ignore trailing bytes (padding). Accepted padding range is 24…64 bytes.

On duplicate rejection, clients SHOULD rotate cliPub.

On failure, server serves only decoy content.

5.3 Noise XK Handshake & Inner Keys (with PQ)
Inner handshake MUST be Noise XK over the outer TLS tunnel.

From 2027-01-01, initiators MUST use hybrid (X25519-Kyber768); prior to that, X25519 is RECOMMENDED.

Derive K0 = HKDF-Expand-Label(TLS-Exporter, "htx inner v1", "", 64).

Split per direction: K0c, K0s.
Derive per-direction nonce salt NS = HKDF(K0*, "ns", "", 12).
AEAD Nonce: nonce = NS XOR (LE64(counter) ‖ LE32(0)); counter starts at 0 and increments per frame.

Rekeying (MUST meet all):

Send KEY_UPDATE when any: ≥ 8 GiB, ≥ 2¹⁶ frames, or ≥ 1 h since last (re)key per direction.
New keys: K' = HKDF(K, "next", transcript_hash, 64); reset counter; derive NS'.
Ordering: Receivers MUST accept KEY_UPDATE out-of-order relative to data frames and MUST discard frames that verify only under the previous key after receiving and acknowledging KEY_UPDATE. Senders MUST cease using the old key immediately after transmitting KEY_UPDATE.

5.4 Inner Frame Format
struct Frame {
  uint24  length;     // ciphertext length (excl. tag)
  uint8   type;       // 0=STREAM, 1=PING, 2=CLOSE, 3=KEY_UPDATE, 4=WINDOW_UPDATE
  varint  stream_id;  // present if type==STREAM or type==WINDOW_UPDATE
  uint8[] ciphertext;
}
Client streams odd; server streams even.
Flow-control window: 65 535; send WINDOW_UPDATE when ≥ 50 % consumed.
5.5 HTTP/2 / HTTP/3 Behaviour Emulation (Adaptive)
H2 SETTINGS MUST mirror origin within tolerances learned during pre-flight (§5.1).
PING cadence MUST be random in [10 s, 60 s] with ±10 % jitter; periodicity is PROHIBITED.
PRIORITY frames SHOULD follow the origin’s baseline rate; if unknown, send on [0.5 %, 3 %] of connections at random.
Idle padding: if no DATA for [200 ms, 1 200 ms], send [0, 3 KiB] dummy encrypted DATA (uniform in both).
5.6 UDP Variant & Anti-Correlation Fallback
Attempt QUIC v1 on UDP-443 with MASQUE CONNECT-UDP.
On failure, retry TCP with randomized back-off [200 ms, 1 200 ms], fresh ClientHello randomness, no session resumption, and fresh QUIC CIDs when applicable.
To defeat induced linkability, clients MUST launch cover connections to ≥ 2 unrelated origins (non-HTX) within [0, 1 000 ms] of the retry; HTX start MUST be delayed by an additional [100, 700 ms] chosen independently.
Cover connections MUST NOT exceed 2 retries per minute and MUST be torn down within [3, 15] s unless they carry user traffic.
6 Overlay Mesh (L3)
6.1 Peer Identity
PeerID = multihash 0x12 0x20 || SHA-256(pubkey).

6.2 Transports
/betanet/htx/1.1.0      (TCP-443)
/betanet/htxquic/1.1.0  (QUIC-443)
/betanet/webrtc/1.0.0   (optional)
6.3 Bootstrap Discovery (Rotating, PoW-Bound)
Clients MUST iterate methods a → e until ≥ 5 peers respond:

Order	Method	Central infra?
a	Rotating Rendezvous DHT: 64 ephemeral IDs SHA256("bn-seed" ‖ BeaconSet(epochDay) ‖ i)	No
b	mDNS service _betanet._udp	No
c	Bluetooth LE UUID 0xB7A7	No
d	Onion v3 list (signed, mirrored via IPFS)	Minimal
e	DNS fallback list	Yes (fallback)
Deterministic seeds from 1.0 are REMOVED.
Responders MUST require proof-of-work (initial ≥ 22 bits, adaptive per §6.5) and rate-limit per source prefix; verification MUST be constant-time.
Epoch definition: epochDay = floor(unixTime/86 400) in UTC.
6.4 Block Exchange
CID = multihash(SHA-256(content)).
Bitswap-v2 on /betanet/bitswap/2.2.0.
Requesters SHOULD open ≥ 2 parallel streams on distinct SCION paths and MAY open a third under good conditions.
6.5 Adaptive Anti-Abuse
Each bootstrap responder MUST maintain sliding-window metrics and adjust PoW difficulty to keep accept rate at the 95th percentile of capacity.
Rate-limits MUST apply per /24 IPv4, /56 IPv6, and per-AS aggregates; any bucket MUST NOT exceed 5 % of responder capacity.
7 Privacy Layer (L4)
7.1 Modes
Mode	Requirement
strict	Every stream through ≥ 3 Nym hops
balanced (default)	≥ 2 hops until peer-trust ≥ 0.8
performance	No mixnet unless destination label .mixreq
7.2 Mixnode Selection (BeaconSet + Per-Stream Entropy)
epoch = floor(unixTime/3600).
BeaconSet(epoch) = XOR32(drand(epoch), nistRBv2(epoch), ethL1_finalized_hash(epoch)), each a 32-byte value; components MAY be substituted by governance.
If all components are unavailable, use fallback BeaconSet(epoch) = SHA256("bn-fallback" ‖ K0c ‖ uint64_be(epoch)) and MUST log the condition.
For each stream, initiator picks 16-B streamNonce.
seed = SHA256( BeaconSet(epoch) ‖ srcPeerID ‖ dstPeerID ‖ streamNonce ).
Hops chosen by VRF over seed from the advertised mixnode set.
Diversity: within (src,dst,epoch), avoid reusing the exact hop set until ≥ 8 distinct sets are tried.
Topology: include at least one hop outside both source and destination AS groups.
7.3 Peer-Trust (for “balanced”)
Computed from:

uptime attestations signed by ≥ 8 distinct AS groups over a 30-day window,
observed relay behaviour,
staked ecash capped by per-AS limits (L7 §10.2). Thresholds are normative; combination is implementation-defined.
8 Naming & Trust (L5)
8.1 Self-Certifying ID
betanet://[/resource]
Verify that the peer’s presented pubkey hashes to the ID.

8.2 Human-Readable Alias Ledger (Finality-Bound 2-of-3 with Liveness)
A record is valid iff an identical payload hash appears finalized on at least 2 of 3 chains, each with native finality:

Handshake L1: ≥ 12 confirmations and not reorged for ≥ 1 h.
Filecoin FVM: chain reports finalized.
Ethereum L2 “Raven-Names”: block marked finalized by the rollup.
Record payload (UTF-8):

betanet1 pk= seq= sig= exp=
seq MUST increase monotonically per pk.
Conflicts: higher seq wins once finality condition is met.
Liveness rule. If fewer than 2 chains provide finality for ≥ 14 days, nodes MAY accept an Emergency Advance for a pk when all hold:

A quorum certificate with ≥ 67 % of effective governance weight (§10.2–§10.3) over the payload hash (pk,seq,sig,exp).
The certificate is anchored once on any available chain (best-effort).
When 2-of-3 finality resumes, the first finalized record at ≥ seq supersedes emergency records.
Quorum certificate format (CBOR map).

{ payloadHash, epoch, signers[], weights[], sigs[] }
Each sig is Ed25519 over ("bn-aa1" ‖ payloadHash ‖ epoch). Verifiers MUST validate weights per §10.2–§10.3 and reject duplicates or lower-epoch certificates.

9 Payment System (L6)
9.1 Federated Cashu Mints
Each mint = FROST-Ed25519 (n ≥ 5, t = 3) group.
Keyset ID = SHA-256(sorted pubkeys).
Mints announce on betanet.mints with ≥ 22-bit PoW and an HTX contact endpoint.
Voucher (128 B):

keysetID32 ‖ secret32 ‖ aggregatedSig64
aggregatedSig64 is the 64-B Ed25519 aggregate signature over secret32.
Relays MUST accept vouchers only for known keysets; unknown keysets MAY be cached pending validation.
Per-keyset and per-peer rate-limits MUST apply.
9.2 Settlement
Relays MAY redeem ≥ 10 000 sat via their own Lightning node or swap with peers. Vouchers MUST NOT leave encrypted streams.

10 Governance & Versioning (L7)
10.1 Node Uptime Score
score = log2(1 + seconds_uptime / 86_400)   // capped at 16
10.2 Voting Power & Anti-Concentration
vote_weight_raw = uptime_score + log10(total_ecash_staked / 1_000 sat + 1)
Per-AS cap: the sum of vote_weight_raw across all nodes within the same L1 AS MUST be capped to 20 % of the global total.
Per-Org cap: nodes mapped to the same RPKI organisation (or equivalent attestation) MUST be capped to 25 % combined.
Effective weight: vote_weight = min(vote_weight_raw, caps).
10.3 Quorum, Diversity & Partition Safety
A proposal passes when all hold:

Σ weight(ACK) ≥ 0.67 × Σ weight(active_nodes_14d), where active_nodes_14d are nodes seen on HTX within 14 days.
ACKs span ≥ 24 distinct AS groups and ≥ 3 SCION ISDs; no single AS contributes > 20 % nor single Org > 25 % of ACK weight.
ACKs are observed over ≥ 2 disjoint path classes per §4 with consistent reachability (median loss < 2 %).
A partition check confirms the median path diversity and ACK composition did not degrade by > 20 % in the 7 days prior to close.
10.4 Upgrade Delay
After threshold, activation waits ≥ 30 days. If §10.3 fails at any time ≥ 7 days before activation, activation MUST be deferred until criteria are met for 7 consecutive days. Raven Development Team publishes a time-lock hash of the final text.

11 Compliance Summary
An implementation is compliant if it:

Implements HTX over TCP-443 and QUIC-443 with origin-mirrored TLS + ECH; performs per-connection calibration (§5.1).
Uses negotiated-carrier, replay-bound access tickets (§5.2) with variable lengths and rate-limits.
Performs inner Noise XK with key separation, nonce lifecycle, and rekeying (§5.3); uses hybrid X25519-Kyber768 from 2027-01-01.
Emulates HTTP/2/3 with adaptive cadences and origin-mirrored parameters (§5.5).
Bridges non-SCION links by HTX-tunnelled transition; no on-wire transition header on public networks (§4.2).
Offers /betanet/htx/1.1.0 and /betanet/htxquic/1.1.0 transports (§6.2).
Bootstraps via rotating rendezvous IDs derived from BeaconSet with PoW and multi-bucket rate-limits; deterministic seeds not used (§6.3–§6.5).
Selects mixnodes using BeaconSet randomness with per-stream entropy and path diversity (§7.2); “balanced” mode enforces ≥ 2 hops until trust ≥ 0.8 (§7.1–§7.3).
Verifies alias ledger with finality-bound 2-of-3 and applies Emergency Advance liveness only under §8.2 conditions; validates quorum certificates as specified.
Accepts 128-B Cashu vouchers for known keysets with PoW adverts and rate-limits; supports Lightning settlement (§9).
Enforces anti-concentration caps, diversity, and partition checks for governance (§10).
Implements anti-correlation fallback with cover connections on UDP→TCP retries (§5.6).
Builds are reproducible and publish SLSA 3 provenance artifacts for release binaries.
12 Interoperability Notes (1.0 Compatibility)
1.1 peers MAY offer /betanet/htx/1.0.0 ALPN for legacy interop.
Legacy on-wire transition headers MUST NOT be used on public networks; bridge via HTX tunnels.
64-B vouchers MAY be issued only to legacy peers; 1.1 receivers MUST accept both for the operator-defined deprecation window.
13 End of Betanet Specification 1.1
