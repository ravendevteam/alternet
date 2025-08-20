# Betanet Protocol Version 1.2

Abstract

This document defines the Betanet Protocol version 1.2, a complete, implementable replacement for the web layer (HTTP and DNS) that provides censorship resistance at the network level, strong user privacy, and decentralized operation with no single authority or central point of failure.

---

Table of Contents

- [1. Introduction](#sec-1)
- [2. Conventions and Terminology](#sec-2)
- [3. Threat Model](#sec-3)
- [4. Architecture and Layering](#sec-4)
- [5. Versioning and Capability Negotiation](#sec-5)
  - [5.1 Version Numbers](#sec-5-1)
  - [5.2 Protocol Identifiers](#sec-5-2)
  - [5.3 Negotiation Procedure](#sec-5-3)
  - [5.4 Downgrade Resistance](#sec-5-4)
- [6. Common Wire Encoding](#sec-6)
  - [6.1 Primitive Types](#sec-6-1)
  - [6.2 Deterministic CBOR](#sec-6-2)
  - [6.3 Timestamps](#sec-6-3)
  - [6.4 Ticket Header ABNF](#sec-6-4)
- [7. Path Selection Layer (L1)](#sec-7)
  - [7.1 Node Identity and Addressing](#sec-7-1)
  - [7.2 Packet Header](#sec-7-2)
  - [7.3 Path Segment and Signature Context](#sec-7-3)
  - [7.4 Multi-Path](#sec-7-4)
  - [7.5 Replay and Freshness](#sec-7-5)
  - [7.6 Encrypted Path Metadata](#sec-7-6)
  - [7.7 Routing and Path Selection (Normative)](#sec-7-7)
- [8. Cover Transport Layer (L2)](#sec-8)
  - [8.1 Outer Transport and Indistinguishability](#sec-8-1)
    - [8.1.1 Origin Template Discovery](#sec-8-1-1)
    - [8.1.2 Front Origin Registry and Compliance Profiles](#sec-8-1-2)
    - [8.1.3 HTTP/2 Behavioral Profile (Normative)](#sec-8-1-3)
    - [8.1.4 HTTP/3 Behavioral Profile (Normative)](#sec-8-1-4)
    - [8.1.5 Compliance Window and Rounding Rules](#sec-8-1-5)
    - [8.1.6 ClientHello Construction](#sec-8-1-6)
    - [8.1.7 Idle and PING Cadence](#sec-8-1-7)
  - [8.2 Inner Channel Overview](#sec-8-2)
    - [8.2.1 AEAD and Noise Suite Fixed Values](#sec-8-2-1)
    - [8.2.2 Complete Noise XK Handshake](#sec-8-2-2)
  - [8.3 Key Schedule](#sec-8-3)
  - [8.4 Frames](#sec-8-4)
    - [8.4.1 Frame Layout and AAD Semantics](#sec-8-4-1)
    - [8.4.2 STREAM](#sec-8-4-2)
    - [8.4.3 WINDOW_UPDATE](#sec-8-4-3)
    - [8.4.4 PING](#sec-8-4-4)
    - [8.4.5 KEY_UPDATE (Concurrency)](#sec-8-4-5)
    - [8.4.6 CLOSE](#sec-8-4-6)
  - [8.5 Indistinguishability Compliance Requirements](#sec-8-5)
- [9. Overlay Mesh Layer (L3)](#sec-9)
  - [9.1 Peer Identity and Multihash Encoding (Normative, Self-Contained)](#sec-9-1)
  - [9.2 Transport Multistream IDs](#sec-9-2)
  - [9.3 Handshake and Capability Exchange](#sec-9-3)
  - [9.4 Multiplexing and Stream ID Allocation](#sec-9-4)
  - [9.5 Peer Discovery and Bootstrap](#sec-9-5)
- [10. Privacy Hop Layer (L4)](#sec-10)
  - [10.1 Relay Selection Lottery](#sec-10-1)
  - [10.2 Relay Mix Capability](#sec-10-2)
  - [10.3 Relay Delay Model](#sec-10-3)
  - [10.4 Relay Forwarding Semantics](#sec-10-4)
- [11. Naming and Trust (L5)](#sec-11)
  - [11.1 Self-Certifying Names](#sec-11-1)
  - [11.2 Alias Ledger (Optional)](#sec-11-2)
- [12. Payments (L6) (Optional)](#sec-12)
  - [12.1 Voucher Format](#sec-12-1)
  - [12.2 Voucher Redemption Service API (Normative)](#sec-12-2)
  - [12.3 BLS Parameters](#sec-12-3)
  - [12.4 Applicability](#sec-12-4)
- [13. Application Layer (L7)](#sec-13)
  - [13.1 BAR: Betanet Application Request](#sec-13-1)
  - [13.2 Mandatory Methods and Status Codes](#sec-13-2)
  - [13.3 Streaming and Backpressure](#sec-13-3)
- [14. Translation Layer for v1.1 Interoperability](#sec-14)
  - [14.1 Overview](#sec-14-1)
  - [14.2 Discovery](#sec-14-2)
  - [14.3 Mapping](#sec-14-3)
  - [14.4 Security](#sec-14-4)
- [15. Error Handling](#sec-15)
- [16. Registries (Canonical)](#sec-16)
  - [16.1 Registry Envelope and Hashing](#sec-16-1)
  - [16.2 Established Registries](#sec-16-2)
- [17. Security Considerations](#sec-17)
- [18. Operational Considerations](#sec-18)
- [19. Compliance Profiles](#sec-19)
- [20. Test Procedures](#sec-20)
- [21. References](#sec-21)
- [Appendix A. ABNF and Data Schemas](#app-a)
- [Appendix B. State Machines](#app-b)
- [Appendix C. Example Encodings and Test Vectors](#app-c)
- [Appendix D. Pseudocode Routines](#app-d)

---

<a id="sec-1"></a>
## 1. Introduction

Betanet 1.2 replaces the traditional web layer with a censorship-resistant and privacy-preserving protocol stack. The design uses indistinguishable outer transport and an authenticated, encrypted inner channel; decentralized discovery; self-certifying identities; and optional privacy hops. The protocol is specified to enable independent, interoperable implementations without out-of-band coordination.

<a id="sec-2"></a>
## 2. Conventions and Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 and RFC 8174 when, and only when, they appear in all capitals, as shown here.

Byte order: Unless otherwise stated, all multi-octet integers are unsigned big-endian. Bit numbering in diagrams starts at 0 (MSB).

Deterministic serialization: When CBOR is used, it MUST be encoded deterministically as defined in Section 6.2.

Terms:
- Node: A software agent implementing Betanet.
- Peer: A remote Node.
- Session: A cryptographic association between two Nodes.
- Relay: A privacy hop in L4.
- Alias: A human-meaningful label bound to a Node identity.
- Voucher: A short payment credential for L6.
- TemplateID: SHA-256 hash of observed origin transport parameters.

<a id="sec-3"></a>
## 3. Threat Model

Betanet assumes adversaries may be:
- Passive on-path observers capable of total traffic capture.
- Active on-path manipulators (packet inject, drop, modify).
- Routing-level adversaries capable of filtering and partitioning.
- Powerful censors controlling name resolution and L3 or L4 filtering.
- Global correlators performing traffic analysis across vantage points.

Trust assumptions:
- No single authority is trusted for operation or naming.
- Nodes do not trust relays or peers with content confidentiality.
- Nodes assume local clocks may drift by up to plus or minus 120 seconds.

Security objectives:
- Full compliance with The Raven Code.
- Confidentiality, integrity, and replay protection for all L2 frames.
- Metadata minimization and cover behavior to resist flow correlation.
- Agility and downgrade resistance across all negotiated features.

<a id="sec-4"></a>
## 4. Architecture and Layering

Layers and responsibilities:

- L0  Access Media: bearer-agnostic.
- L1  Path Selection: addressing, routing, and path validation.
- L2  Cover Transport: indistinguishable outer transport with an inner authenticated encryption channel.
- L3  Overlay Mesh: peer discovery, session management, and stream multiplexing.
- L4  Privacy Hop: optional relay to add mixing latency and unlinkability.
- L5  Naming and Trust: self-certifying identities with optional aliasing.
- L6  Payments: vouchers and settlement hooks.
- L7  Applications: web-layer replacement services.

Each layer is independently versioned and negotiated as defined in Section 5. Downward references are informative; the wire-visible fields are defined in their owning sections.

<a id="sec-5"></a>


**Canonical wire stack mapping (Normative).**

- Outer transport: **TLS 1.3 over TCP** or **QUIC v1 over UDP** to a real front origin selected via Section 8.1.1.
- Application cover: **HTTP/2 or HTTP/3** behavior precisely as profiled in Sections 8.1.3 and 8.1.4.
- Inner authenticated channel: a Noise XK channel (Section 8.2) that carries **L2 Frames** (Section 8.4).
- L1 Path Selection is **not** a separate on-the-wire layer. L1 structures are encoded as **L2 control frames** and are not independently visible to the network.

**Layering rule (Normative).** All path metadata and routing controls defined in Section 7 are serialized as the payload of L2 `PATH_CONTROL` frames (Type = `0x20`) and thus protected by the inner channel AEAD. L2 frames are carried inside the HTTP/2 or HTTP/3 message bodies/tunnels established over TLS/QUIC. There is no mapping in which “L1 encapsulates L2”.

**Carriage mapping.**
- L1 “Packet Header” (Section 7.2) is the **binary payload** of L2 `PATH_CONTROL` and never appears as a stand‑alone datagram on the wire.
- Application data is carried in L2 `STREAM` frames (Section 8.4.2).
- Flow control and credit signals use L2 `WINDOW_UPDATE` (Section 8.4.3).
- Congestion feedback from L1 (CF) is conveyed using L2 `PATH_CONTROL` subtype `CF` (Section 7.7).

Conformance: any implementation that exposes an L1 header directly on the wire is **non‑conformant**.

## 5. Versioning and Capability Negotiation

<a id="sec-5-1"></a>
### 5.1 Version Numbers

A version is MAJOR.MINOR.PATCH. This document defines MAJOR=1, MINOR=2, PATCH=0.

<a id="sec-5-2"></a>
### 5.2 Protocol Identifiers

Each layer exposes a Protocol ID string on L3 capability exchange:
- L2: "betanet/htx/1.2.0"
- L3: "betanet/mesh/1.2.0"
- L4: "betanet/mix/1.2.0"
- L5: "betanet/naming/1.2.0"

<a id="sec-5-3"></a>
### 5.3 Negotiation Procedure

During L3 handshake (Section 9.3), peers send a DET-CBOR map:
{"l2":[strings], "l3":[strings], "l4":[strings], "l5":[strings], "features":{string: bool}}

Peers MUST select the highest mutually supported ID per layer. The selected IDs are bound into the L2 key schedule exporter context (Section 8.3). A peer detecting a version mismatch without a translation path MUST abort with error VERSION_NEGOTIATION_FAILED (Section 15).

<a id="sec-5-4"></a>
### 5.4 Downgrade Resistance

The L2 inner key derivation (Section 8.3) includes the concatenation of both peers offered capability sets and the final chosen IDs in the exporter context. If a middlebox strips capabilities, keys will differ and decryption will fail.

<a id="sec-6"></a>
## 6. Common Wire Encoding

<a id="sec-6-1"></a>
### 6.1 Primitive Types

- u8, u16, u24, u32, u48, u64: Unsigned integers, big-endian.
- bstr[N]: Fixed-length byte string of length N.
- varstr: Length-prefixed (u16) byte string.
- varvec: Length-prefixed (u16) vector of items of the same type.

<a id="sec-6-2"></a>
### 6.2 Deterministic CBOR (Normative, Self-Contained)

All CBOR used by this specification MUST be encoded deterministically as defined here.

1) General
- Indefinite-length items are PROHIBITED.
- Floating-point numbers, tags, and simple values other than true, false, null are PROHIBITED. Implementations MUST NOT emit or accept them on the wire.
- Encoding MUST be the shortest form that represents the value.

2) Unsigned and Negative Integers
- Major type 0 encodes n >= 0; major type 1 encodes -1-n for negative integers.
- Additional information 0..23 encodes the value directly; 24,25,26,27 use 1,2,4,8 additional bytes respectively. Shortest-length rule MUST be applied.

3) Byte Strings (bstr) and Text Strings (tstr)
- Must use definite-length forms only.
- Length itself MUST use shortest-length encoding.
- Text strings MUST be well-formed UTF-8; invalid sequences are REJECTED.

4) Arrays
- Definite-length only; length MUST be encoded in shortest-length form.
- Elements are encoded in order.

5) Maps
- Definite-length only; length MUST be encoded in shortest-length form.
- Keys MUST be unique under bytewise equality of their encoded forms.
- Entry ordering: sort by the bytewise lexicographic order of the deterministically encoded key. Compare as raw bytes; do not apply type-aware collation.

6) Canonical Encoding Function (Pseudocode)
See Appendix D.

7) Deterministic CBOR for TemplateID Context
Maps used in key schedules and TemplateID computation MUST follow the above rules. The SHA-256 over the exact encoded bytes is the value used elsewhere in this specification.

Unless otherwise specified, integrity is provided by AEAD tags. No additional checksums are used.

<a id="sec-6-3"></a>
### 6.3 Timestamps

Timestamps are u64 milliseconds since UNIX epoch. Nodes MUST accept up to plus or minus 120000 ms skew unless a section mandates stricter bounds.

<a id="sec-6-4"></a>

### 6.4 Ticket Header ABNF

The Betanet ticket is conveyed using an HTTP Structured Header (RFC 8941) on cover requests and responses.

Field name: `BN-Ticket`

ABNF:

```
Ticket-Field = sf-dictionary
; MUST contain exactly: v, tok
; MAY contain: ctx

v = sf-token ; MUST be "v1"
tok = sf-binary ; fixed length 120 bytes after base64url decoding
ctx = sf-string ; optional context label ≤32 characters
```

Constraints (Normative):
- The total header size (name + value) MUST NOT exceed 256 bytes.
- `v` MUST equal `v1`.
- `tok` MUST decode (base64url without padding) to exactly 120 bytes.
- `tok` MUST be produced as: `tok = HKDF-Expand(ikm=exporter_secret, info="BN-Ticket v1" || ctx, L=120)` where `exporter_secret` is the L2 exporter for the session (Section 8.3).
- Implementations MUST send `tok` padded using base64url **without** `=` padding, and recipients MUST reject inputs containing `=`.
- `ctx` if present is ASCII alphanumerics plus `-` and `_`.

Generation:
- A sender MUST generate a fresh `tok` per HTTP request that carries an inner-channel message, and reuse is forbidden.
- Receivers MUST verify length and base64url alphabet and MUST NOT attempt to infer entropy from `tok`.

Non-compliance:
- If the header is missing or malformed, the connection MUST be closed with error `L2_TICKET_INVALID`.
## 7. Path Selection Layer (L1)

<a id="sec-7-1"></a>
### 7.1 Node Identity and Addressing

NodeID = SHA-256(Ed25519 public key) truncated to 160 bits.
On-wire encoding is 20-octet bstr.

<a id="sec-7-2"></a>
### 7.2 Packet Header

```
0               1               2               3
+---------------+---------------+---------------+---------------+
| Version (8)   | Type (8)      | Header Len (16)               |
+---------------+---------------+---------------+---------------+
| Payload Len (16)              | Flags (8)     | SegCnt (8)    |
+---------------+---------------+---------------+---------------+
|        Path Segments (variable, see 7.3)                      |
+---------------------------------------------------------------+
```

Version: 0x02.  
Type: 0x01 single-path, 0x03 multi-path list.  
Flags: bit0=EncryptedPathMetaPresent, bit1=CongestionFeedback.  
SegCnt: Number of segments. MUST be >= 1 for Type 0x01.

Payload Len denotes the exact number of octets of the L1 logical header structure. This structure is carried verbatim as the payload of the L2 `PATH_CONTROL` frame (Type 0x20). There is no stand‑alone L1 packet on the wire and therefore no L1 fragmentation or aggregation.

<a id="sec-7-3"></a>
### 7.3 Path Segment and Signature Context

Each segment encodes:

```
+----------------+----------------+----------------+----------------+
| NextNode (20B) | Expiry (u32)   | Timestamp (u64)| SigLen (u16)   |
+----------------+----------------+----------------+----------------+
| Sig (SigLen)                                                   |
+----------------------------------------------------------------+
```

Sig is Ed25519 over H where:
H = SHA-256("BN-L1-seg" || Version || Type || Flags || NextNode || Expiry || PrevNode || Timestamp)

PrevNode is the sender NodeID (20B). A Node MUST verify Sig before forwarding. Failure: PATH_INVALID (Section 15).

<a id="sec-7-4"></a>
### 7.4 Multi-Path

Type 0x03 carries SegCnt >= 2 segments. The sender selects one based on local policy. Receivers MAY send CongestionFeedback frames to influence future selection (Section 7.6).

<a id="sec-7-5"></a>
### 7.5 Replay and Freshness

A Node MUST reject segments with Expiry < now-120000 ms or future Expiry > now+3600000 ms. Nodes MUST maintain a 10 minute cache of (PrevNode, NextNode, Expiry, Timestamp) to reject duplicates.

<a id="sec-7-6"></a>
### 7.6 Encrypted Path Metadata

If Flags bit1=1, an L1 CF structure follows the last segment:

```
+------------+------------+------------+------------+
| ECN (u8)   | LossPct(u8)| RTTms(u16) | Spare(u16) |
+------------+------------+------------+------------+
```

Receivers MAY use CF to adjust path selection. CF is authenticated by L2 AEAD and need not be separately signed.

<a id="sec-8"></a>


<a id="sec-7-7"></a>
### 7.7 Routing and Path Selection (Normative)

Provide a mandatory, interoperable algorithm for next-hop selection, path construction, liveness, loop avoidance, and use of congestion feedback (CF).

**Path construction.**
- Clients build paths of 3 segments by default: entry → core → exit. Each segment is a NextNode (Section 7.3) with Expiry ≥ now()+120s.
- Nodes MUST refuse to construct or forward paths containing repeated NodeIDs (loop prevention).

**Next-hop selection.**
- For each segment position, choose uniformly from peers advertising `role ∈ {entry, core, exit}` and `health=OK` within the last 60s.
- Exclude peers with failure-rate >10% in the last 5 minutes or with CF.congestion=SEVERE.
- ECMP: if multiple candidates remain, pick the one with the lowest moving RTT; ties broken randomly.

**Liveness and health.**
- A node MUST probe a next-hop if idle for 20s by sending `PATH_CONTROL/PROBE` and waiting ≤1 RTT for `PROBE_ACK`. On timeout, mark the peer `health=STALE` and attempt one alternate hop.
- Failure detection: if 3 consecutive sends to a hop time out, evict it from the candidate set for 5 minutes (exponential backoff with jitter in [0.5×,1.5×]).

**Congestion feedback (CF).**
- Each hop MAY attach a CF record indicating `queue_depth (u16)`, `rtt_sample (u16 ms)`, and `congestion ∈ {NONE, MODERATE, SEVERE}`.
- Senders MUST throttle new path construction through hops reporting SEVERE by halving their selection probability for 60s.

**Forwarding rule.**
- Upon receipt of a `PATH_CONTROL` directing a next-hop, a node MUST forward the associated L2 `STREAM` frames to that hop. Header fields a node may mutate: `Timestamp` (fresh) and `Sig` (recomputed over the segment it owns). No other fields may be modified.

**Error propagation.**
- On forwarding failure, generate `PATH_CONTROL/ERROR` with `code ∈ {NO_ROUTE, PEER_DOWN, TTL_EXPIRED, POLICY_DENY}` and send it back along the reverse of the known segments. Intermediaries MUST NOT forge errors.

**Timers.**
- Segment TTL default: 120s; max: 600s.
- Per-hop send timeout: 3× smoothed RTT (min 200ms, max 3s).

## 8. Cover Transport Layer (L2)

<a id="sec-8-1"></a>
### 8.1 Outer Transport and Indistinguishability

L2 runs inside a real TLS 1.3 or QUIC v1 session to indistinguishably mimic ordinary web traffic. The client MUST calibrate its TLS ClientHello and QUIC transport parameters against the chosen origin template.

<a id="sec-8-1-1"></a>
#### 8.1.1 Origin Template Discovery

Before establishing the Betanet session to a front origin, the client performs a calibration fetch (HTTPS GET "/robots.txt" or "/favicon.ico") to obtain the origin observable parameters:
- TLS: cipher suites, extensions order, ALPN list, supported_groups, signature_algorithms, GREASE presence.
- HTTP/2 SETTINGS and frame behaviors if ALPN is "h2".
- QUIC: initial_max_data, active_connection_id_limit, max_idle_timeout.

The client computes TemplateID = SHA-256 over a deterministic CBOR encoding of these parameters (Section 6.2) and stores it for 24 hours. If the origin template changes within the cache period, the client MUST refresh and recompute TemplateID before initiating new sessions.

<a id="sec-8-1-2"></a>
#### 8.1.2 Front Origin Registry and Compliance Profiles

A deployment MAY maintain a signed registry of admissible front origins to enable interoperable calibration across vendors. Each entry contains:
- TemplateID: SHA-256 of the canonical origin parameter set
- HostPattern: glob or regular-expression for acceptable hostnames
- ComplianceProfile: "STANDARD" or "EXTENDED"
- Expiry: absolute UTC timestamp

Implementations MUST accept any locally configured origin, but SHOULD consult a registry when available for cross-vendor interoperability. Registry records are stored in the alias-ledger with type "FrontOriginRegistry".

<a id="sec-8-1-3"></a>
#### 8.1.3 HTTP/2 Behavioral Profile (Normative)

When ALPN negotiates "h2", clients and servers MUST conform to the following minimum HTTP/2 behaviors during origin calibration:

1) Connection Preface
- Client MUST send the literal connection preface: "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".
- Immediately follow with a SETTINGS frame.

2) SETTINGS Frame
- Frame format: Length(24) | Type=0x4 | Flags(8) | StreamID=0x0.
- Payload is a sequence of 6-byte parameters: Identifier(16) | Value(32).
- REQUIRED identifiers and default values unless overridden by origin template:
  - SETTINGS_ENABLE_PUSH (0x2): 0
  - SETTINGS_MAX_CONCURRENT_STREAMS (0x3): implementation-defined
  - SETTINGS_INITIAL_WINDOW_SIZE (0x4): 65535
  - SETTINGS_MAX_FRAME_SIZE (0x5): 16384
  - SETTINGS_MAX_HEADER_LIST_SIZE (0x6): implementation-defined
- A peer that receives SETTINGS with the ACK flag set MUST NOT apply new values.

3) Simple Request for Calibration
- Use a single GET request for "/robots.txt" or "/favicon.ico" on stream 1.
- Header compression via HPACK with a fresh dynamic table; do not send PRIORITY.
- The client MUST send HEADERS (END_HEADERS=1, END_STREAM=1) followed by no DATA frames.

4) Frame Constraints
- Allowed during calibration: SETTINGS, WINDOW_UPDATE, PING, GOAWAY, HEADERS, DATA.
- PROHIBITED: PRIORITY frames and PUSH_PROMISE.
- Stream 0 is used only for connection-level frames.

5) TemplateID Construction
- Canonicalized parameter set includes: ordered ALPN list, GREASE presence, TLS groups, signature algorithms, HTTP/2 SETTINGS identifier/value pairs (ordered by identifier), QUIC transport parameters when ALPN is "h3", and any server-advertised limits discovered during the calibration fetch.
- Encode this parameter set using Deterministic CBOR (Section 6.2) and compute TemplateID = SHA-256 over the resulting bytes.

<a id="sec-8-1-4"></a>
#### 8.1.4 HTTP/3 Behavioral Profile (Normative)

When ALPN negotiates "h3", clients and servers MUST conform to QUIC v1 defaults and template-aligned transport parameters. initial_max_data, active_connection_id_limit, and max_idle_timeout MUST be within the template ranges recorded in TemplateID.

<a id="sec-8-1-5"></a>
#### 8.1.5 Compliance Window and Rounding Rules

Implementations MUST keep negotiated parameters within plus or minus 10 percent of the template values unless the origin rejects such parameters, in which case clients MUST retry with exact matches.

<a id="sec-8-1-6"></a>
#### 8.1.6 ClientHello Construction

The client MUST construct ClientHello to match the template exactly, including GREASE. 0-RTT MUST be disabled. ALPN MUST offer the exact template order. QUIC transport parameters MUST be within plus or minus 10 percent of observed template values unless the origin rejects such parameters, in which case the client MUST retry with exact matches.

<a id="sec-8-1-7"></a>
#### 8.1.7 Idle and PING Cadence

Idle and PING cadence MUST be within plus or minus 10 percent of the template observed cadence.

<a id="sec-8-2"></a>
### 8.2 Inner Channel Overview

After outer TLS or QUIC is established, peers run a Noise-based inner handshake providing mutual authentication and forward secrecy. The inner channel carries framed data for L3 streams.

<a id="sec-8-2-1"></a>
#### 8.2.1 AEAD and Noise Suite Fixed Values

The inner channel uses ChaCha20-Poly1305 (key length 32, nonce length 12, tag 16) and SHA-256. The Noise suite string is fixed to:

Noise_XK_25519_ChaChaPoly_SHA256

X25519 is used for ECDH. Ed25519 static keys are converted to and from X25519 using the private scalar clamping defined for X25519.

<a id="sec-8-2-2"></a>
#### 8.2.2 Complete Noise XK Handshake (Normative, Self-Contained)

This section defines the precise state machine used for the inner handshake so that no external Noise document is required. The pattern corresponds to XK, where the initiator knows the responder static public key (rs), and the initiator static key (si) is sent encrypted during the handshake.

Notation:
- DH(a,b)  = X25519(a.private, b.public)
- HKDF     = HKDF-Extract and HKDF-Expand with SHA-256
- AEAD     = ChaCha20-Poly1305 with 12-byte nonce; AAD is the running handshake hash h
- xor(a,b) = bytewise XOR on equal-length byte strings

State variables:
- ck  : 32-byte chaining key
- h   : 32-byte handshake hash
- k   : 32-byte temporary AEAD key or None
- n   : 64-bit little-endian AEAD nonce counter (starts at 0)
- rs  : responder static public key (32 bytes)
- si  : initiator static private key; spki_i = Ed25519 public; x25519(si) used for DH
- sr  : responder static private key (server side)

Initialization (both sides):
- proto = "Noise_XK_25519_ChaChaPoly_SHA256"
- h  = SHA-256(proto)
- ck = h
- Pre-message: initiator knows rs; include it into transcript
- h  = SHA-256(h || rs)

Message 1 (Initiator -> Responder): "e, es"
1. Generate ephemeral key pair (e_i, E_i).
2. h = SHA-256(h || E_i)
3. ck, k_tmp = HKDF(ck, DH(e_i, rs))   ; es
   ck = k_tmp[:32]; k = k_tmp[:32]
4. No payload. Send E_i (32 bytes). Clear k after message 1.
5. Set k = None; n = 0.

Message 2 (Responder -> Initiator): "e, ee, se"
1. Generate ephemeral key pair (e_r, E_r); send E_r.
   h = SHA-256(h || E_r)
2. ck, temp = HKDF(ck, DH(e_r, e_i))   ; ee
   ck = temp[:32]
3. ck, k = HKDF(ck, DH(sr, e_i))       ; se
   ck = k[:32]; k = k[:32]
4. Encrypt empty payload with AEAD(k, n, aad=h) producing tag only; send tag.
   n += 1

Message 3 (Initiator -> Responder): "s, se"
1. ck, k = HKDF(ck, DH(si, e_r))       ; se
   ck = k[:32]; k = k[:32]
2. Payload = si_pub (Ed25519 public key, 32 bytes). If implementations store Ed25519,
   they MUST include the Ed25519 public key as payload; conversion to X25519 is for DH only.
3. Ciphertext = AEAD_Encrypt(k, n, aad=h, plaintext=si_pub); send ciphertext.
   n += 1

Handshake Completion (both sides):
Split to transport secrets:
- ts_c, ts_s = HKDF(ck, "split")
- key_c = HKDF(ts_c, "key")[0:32]; nonce_salt_c = HKDF(ts_c, "nonce")[0:12]
- key_s = HKDF(ts_s, "key")[0:32]; nonce_salt_s = HKDF(ts_s, "nonce")[0:12]
Transport nonces use a per-direction 64-bit counter:
- nonce = NonceSalt_d XOR (u64_le(counter) || 0x00000000)
- counter starts at 0 and increments by 1 per record.

Key Derivation Notes:
- HKDF(ck, input) denotes:
  prk = HKDF-Extract(salt=ck, IKM=input)
  out = HKDF-Expand(prk, info="betanet/noise", L=64)
  return (out[:32], out[32:64])

Validation:
Any decryption failure or unexpected message field terminates the handshake. On failure, both sides MUST erase k, ck, and all ephemeral private keys.

<a id="sec-8-3"></a>
### 8.3 Key Schedule

Let EKM = TLS exporter with label "betanet/1.2/inner" and context:
context = DET-CBOR({"l2":"betanet/htx/1.2.0", "l3":chosen_l3_id, "l4":chosen_l4_id, "l5":chosen_l5_id, "caps_client":client_caps, "caps_server":server_caps, "template":TemplateID})

K0 = HKDF-Extract(salt=0^32, IKM=EKM)  
TrafficSecret_c = HKDF-Expand-Label(K0, "ts_c")  
TrafficSecret_s = HKDF-Expand-Label(K0, "ts_s")

Each direction has:
Key_d = HKDF-Expand-Label(TrafficSecret_d, "key", 32)  
NonceSalt_d = HKDF-Expand-Label(TrafficSecret_d, "nonce", 12)

Nonce = NonceSalt_d XOR (u64_le(counter) || 0x00000000)  
Counter starts at 0 and increments per frame. Counters reset on a successful KEY_UPDATE and MUST NOT wrap.

<a id="sec-8-4"></a>
### 8.4 Frames

<a id="sec-8-4-1"></a>
#### 8.4.1 Frame Layout and AAD Semantics

L2 frames are carried within the inner Noise channel over HTTP/2 or HTTP/3. L1 is not a wire layer and does not encapsulate L2. Framing boundaries are defined entirely by L2; senders and receivers MUST treat L2 frames as self-contained units. 



Frame:

```
+-----------+-----------+-----------+-----------+
| Len(u24)  | Type(u8)  | StreamID(u32)        |
+-----------+-----------+-----------+-----------+
| AAD(u16)  | Ciphertext (variable)            |
+-----------+-----------+-----------+-----------+
| Tag(16)                                      |
+----------------------------------------------+
```

Len counts bytes from Type through Tag inclusive (i.e., excludes the 3-octet Len field itself). The AAD for AEAD is:
AAD = Len(u24) || Type || StreamID || AAD(u16)

Types:
- 0x00 STREAM
- 0x01 WINDOW_UPDATE
- 0x02 PING
- 0x03 KEY_UPDATE
- 0x04 CLOSE

<a id="sec-8-4-2"></a>
#### 8.4.2 STREAM

Payload:

```
+----------------+----------------+----------------+
| Flags(u8)      | Offset(u64)    | Data (varstr) |
+----------------+----------------+----------------+
```

Flags: bit0=FIN.  
Ordering: In-order delivery per StreamID; receivers MAY buffer gaps.  
Flow control: A sender MUST NOT exceed the peer advertised window.

Senders MAY segment application bytes across multiple STREAM frames; segmentation is at L2 (not L1).

<a id="sec-8-4-3"></a>
#### 8.4.3 WINDOW_UPDATE

Payload:

```
+----------------+----------------+
| Scope(u8)      | Credit(u32)    |
+----------------+----------------+
```

Scope: 0=connection-wide, 1=per-stream.  
Credit adds to the corresponding window. Initial connection window is 65535 octets, per-stream window is 32768 octets.

<a id="sec-8-4-4"></a>
#### 8.4.4 PING

Payload: 8-octet opaque token reflected by peer within 5 seconds.

<a id="sec-8-4-5"></a>
#### 8.4.5 KEY_UPDATE (Concurrency)

No payload. Either side MAY initiate at any time. Each direction maintains a generation counter starting at 0. Upon sending KEY_UPDATE, the sender derives next TrafficSecret_d:

TrafficSecret_d' = HKDF-Expand-Label(TrafficSecret_d, "next", 32)

The sender marks next generation as pending. Receivers MUST accept frames under either the current or the next key (at most one pending) with an overlap of up to 3 frames. If both sides update concurrently, both next generations are valid; after 3 successfully verified frames under the new key in each direction, peers MUST drop acceptance of the prior generation. Frames older than the previous generation MUST be discarded as OLD_KEY_FRAME.

<a id="sec-8-4-6"></a>
#### 8.4.6 CLOSE

Payload:

```
+----------------+----------------+
| Code(u16)      | Reason(varstr) |
+----------------+----------------+
```

After sending or receiving CLOSE at connection scope, a peer MUST cease sending frames other than CLOSE and drain for 2 seconds.

<a id="sec-8-5"></a>
### 8.5 Indistinguishability Compliance Requirements

Implementations MUST mimic the chosen TemplateID:
- ALPN order MUST match exactly.
- TLS extensions order and GREASE presence MUST match exactly.
- HTTP/2 SETTINGS values MUST be within plus or minus 10 percent of the template.
- QUIC transport parameters MUST be within plus or minus 10 percent of the template.
- Idle and PING cadence MUST be within plus or minus 10 percent of the template observed cadence.

These requirements are compliance-critical for censorship resistance but do not affect wire compatibility.

<a id="sec-9"></a>


**Signature and envelope (Normative).**
- Registry is a DET-CBOR map with keys: `template_id (bstr[32])`, `host_pattern (tstr)`, `profile (tstr)`, `expiry (u64)`.
- The entire map is signed using Ed25519 over `SHA-256("BN-FOR1" || cbor_bytes)`, producing `sig (bstr[64])` and `signer (bstr[32])`.

**Distribution.**
- Registries MAY be distributed inline with software and/or via the Alias Ledger (Section 11.2) under record type `FrontOriginRegistry`.
- Clients MUST cache entries until `expiry` and MUST refresh on or before expiry; stale entries MUST NOT be used for calibration.

**Update cadence.**
- Operators SHOULD publish updated registries at least every 30 days or immediately upon origin change.

**Failure handling.**
- If calibration produces a `TemplateID` not in the registry, clients MUST either (a) fall back to local policy that allows ad‑hoc acceptance, or (b) abort with `L2_ORIGIN_MISMATCH` depending on the selected `ComplianceProfile`.
- Divergence is detectable: include the `TemplateID` used in the L2 exporter context and surface it in error logs.

**Interoperability.**
- Different vendors observing the same origin MUST compute the same `TemplateID` given Section 8.1.3/8.1.4 and the rounding rules in Section 8.1.5; otherwise the connection is considered non‑compliant.
## 9. Overlay Mesh Layer (L3)

<a id="sec-9-1"></a>
### 9.1 Peer Identity and Multihash Encoding (Normative, Self-Contained)

PeerID is defined as the multihash of the Ed25519 public key using SHA-256.

1) Digest
- Compute digest = SHA-256(Ed25519_public_key_bytes) producing 32 bytes.

2) Varint (LEB128)
- Encode unsigned integers using LEB128 (little-endian base-128).
- For each 7-bit group of the value, emit (group | 0x80), except the last group which is emitted without 0x80.
- Example: 0x12 encodes as the single byte 0x12. The length 32 encodes as 0x20.

3) Multihash Layout
- code   = varint(0x12)          ; sha2-256
- length = varint(32)
- value  = digest
- PeerID = code || length || value

4) String Form
- Unless otherwise specified, PeerIDs are rendered as lowercase hex of the bytes above.
- Implementations MAY also support Base32 lower without padding, prefixed with "mh1-".

<a id="sec-9-2"></a>
### 9.2 Transport Multistream IDs

"/betanet/htx/1.2.0" for L2 over TLS, "/betanet/htxquic/1.2.0" for L2 over QUIC.

<a id="sec-9-3"></a>
### 9.3 Handshake and Capability Exchange

Upon establishing L2, each side opens StreamID=1 and exchanges CapMsg:

CapMsg := DET-CBOR map per Section 5.3.

The side with lexicographically lower PeerID becomes the Decider and selects the final IDs. If PeerIDs compare equal (collision), negotiation MUST abort with AUTH_FAILED. The Decider sends SelMsg:

SelMsg := DET-CBOR {"l2": string, "l3": string, "l4": string, "l5": string}

Both sides commit the selection and update their exporter context (Section 8.3).

<a id="sec-9-4"></a>
### 9.4 Multiplexing and Stream ID Allocation

L3 allocates StreamIDs (odd=client, even=server). StreamIDs are u32 monotonically increasing per opener. A peer MUST NOT reuse IDs.

<a id="sec-10"></a>


<a id="sec-9-5"></a>
### 9.5 Peer Discovery and Bootstrap (Normative)

**Bootstrap sources.**
1) **Static seeds.** Software MUST ship with at least 5 seed peers (PeerID + host:port + certificate pin) maintained by independent operators.
2) **Alias Ledger.** Deployments MAY publish additional bootstrap peers via the Alias Ledger (Section 11.2) record type `BootstrapPeer`.

**Join procedure.**
- A node attempts up to 3 parallel connections to distinct seeds using L2 over TLS and/or QUIC as permitted by policy. No more than one connection per seed at a time.
- For each successful connection, request `L3/DISCOVER` (STREAM type `0x10`) and receive a signed peer list containing: PeerID, addresses, roles, expiry.
- Validate the list signature against the seed’s advertised identity; reject lists older than 10 minutes.

**Gossip.**
- Nodes periodically (every 60–120s, jittered) exchange `L3/GOSSIP` (STREAM type `0x11`) with 32 randomly chosen peers, sending deltas since `LastSeen`.
- Entries expire 10 minutes after `expiry` unless refreshed.

**Admission control.**
- Peers MUST verify that inbound connections present a valid L2 handshake and do not exceed per-IP connection caps (default 8).
- Rate limiting: token bucket per PeerID capacity 100, refill 20/s.

**Backoff.**
- On failure to connect to a seed, use exponential backoff starting at 10s, cap at 10 minutes, full jitter.

Security: all discovery and gossip messages are sent over the inner channel; no unauthenticated UDP/TCP beacons are used.

## 10. Privacy Hop Layer (L4)

<a id="sec-10-1"></a>
### 10.1 Relay Selection Lottery

An L4 Relay is an L3 peer flagged "mix=true" in capabilities. A sender MAY route via up to 3 relays. Each relay only sees adjacent NodeIDs. Relay selection uses a deterministic lottery seeded by SHA-256(SessionKey || timebucket_5min || client_random128), where client_random128 is fresh per session.

<a id="sec-10-2"></a>
### 10.2 Relay Mix Capability

Relays that advertise "mix=true" MUST support delay insertion per Section 10.3 and MUST NOT modify L2 frames.

<a id="sec-10-3"></a>
### 10.3 Relay Delay Model

Minimum delay per relay is 30 to 150 ms jittered; total added delay MUST NOT exceed 900 ms. Implementations SHOULD make relays jurisdictionally and topologically diverse; clients SHOULD rotate selection every 24 hours.

<a id="sec-11"></a>


<a id="sec-10-4"></a>
### 10.4 Relay Forwarding Semantics (Normative)

**Visibility and cryptography.**
- L4 relays MUST NOT decrypt L2 `STREAM` payloads. They MAY read `PATH_CONTROL` headers only to the extent needed to perform delay/mix operations and next-hop forwarding.

**Permitted mutations.**
- Relays MAY update the per-hop `Timestamp` they own and MUST recompute the corresponding per-hop signature (Section 7.3).
- Relays MUST NOT change `NextNode`, `Expiry`, `SegCnt`, or any prior-hop signature fields.

**Delay application.**
- Apply an independent random delay sampled from U[30ms,150ms] per Section 10.3. The cumulative delay budget for a path is bounded to 600ms unless the client explicitly marks a stream as `priority=low`.

**Error handling.**
- On next-hop failure, a relay sends `PATH_CONTROL/ERROR` with `code=PEER_DOWN` toward the previous hop and MUST NOT buffer user payload for more than 2s awaiting recovery.

**Accounting.**
- If payments (L6) are enabled, relays MUST charge per-byte forwarded; accounting records are emitted on `L3/ACCOUNT` streams and are authenticated by the relay’s Ed25519 key.

## 11. Naming and Trust (L5)

<a id="sec-11-1"></a>
### 11.1 Self-Certifying Names

A Name is: "bn1:" base32(NodeID || checksum[4]) where checksum = first 4 bytes of SHA-256("BN-NAME" || NodeID). Names are resolved locally to NodeIDs without external authorities.

<a id="sec-11-2"></a>

### 11.2 Alias Ledger (Optional)

**Record format.**
AliasRecord := DET-CBOR {"alias": tstr, "node": bstr[20], "seq": u64, "sig": bstr[64], "ts": u64}
Sig is Ed25519 over SHA-256("BN-L5" || alias || node || seq || ts).

**Finality and monotonicity (Normative).**
- Two‑of‑three signer finality is REQUIRED per alias for a record to be accepted.
- Nodes MUST reject any record where the per‑alias `seq` is not strictly increasing.

**Bootstrap and trust anchors.**
- Software MUST ship with at least three bootstrap signer public keys (Ed25519) from independent operators. These keys are the initial trust anchors.

**Rotation.**
- Signer rotation is encoded by a `RotateMaintainers` record:
  `RotateMaintainers := DET-CBOR {"type":"rot","new":[bstr[32],...],"prev":[bstr[32],...],"seq":u64,"sig":[bstr[64],bstr[64]]}`
- Exactly two signatures from distinct current maintainers MUST endorse the rotation. After a valid rotation at sequence `seq`, only `new` keys are authoritative.

**Distribution.**
- Alias and rotation records MAY be distributed via:
  (a) inline with software releases, and/or
  (b) over L3 `ALIAS_GOSSIP` streams signed by current maintainers.

**Storage and expiry.**
- Nodes store the latest accepted record per alias plus the last 5 historical records.
- Records with `ts` older than 365 days MUST be ignored unless accompanied by a newer rotation proving continuity.

**Error handling.**
- `ALIAS_SIG_INVALID`, `ALIAS_SEQ_REPLAY`, and `ALIAS_ROTATION_CONFLICT` are fatal to the offending record but MUST NOT tear down the transport.

Gossip: use L3 STREAM type `0x00`; messages are authenticated and carried over the inner channel.
## 12. Payments (L6) (Optional)

<a id="sec-12-1"></a>
### 12.1 Voucher Format

Voucher := bstr[128] structured as: keysetID(32) || secret(32) || aggSig(64). Validation and redemption are out of scope for interop; vouchers are opaque to the transport and are only forwarded.

<a id="sec-12-2"></a>
### 12.2 Voucher Redemption Service API (Normative)

Interoperable deployments SHOULD provide:
1) A Voucher Redemption Service (VRS) reachable over L3 that maps voucher IDs to balances.
2) A signed query and commit API for validity checks and redemption.
3) A PaymentRegistry record type in the alias-ledger that versions the schema.

Nodes that do not implement L6 MUST pass vouchers unchanged.

<a id="sec-12-3"></a>
### 12.3 BLS Parameters

Deployments that implement aggregated signature vouchers MUST publish BLS parameters and keyset IDs in the PaymentRegistry. The exact pairing-friendly curve and serialization are deployment policy and MUST be unambiguous.

<a id="sec-12-4"></a>
### 12.4 Applicability

L6 is OPTIONAL. L2 and L3 behavior MUST NOT change based on L6 presence. Vouchers are opaque and are forwarded without inspection.

<a id="sec-13"></a>
## 13. Application Layer (L7)

<a id="sec-13-1"></a>
### 13.1 BAR: Betanet Application Request

BAR replaces HTTP semantics within Betanet streams.

Request := DET-CBOR {"m": tstr, "u": tstr, "h": {tstr: (tstr or bstr)}, "b": bstr}  
Response := DET-CBOR {"s": u16, "h": {tstr: (tstr or bstr)}, "b": bstr}

Methods are application-defined. Servers MUST treat unknown methods as 400 (Bad Request).

<a id="sec-14"></a>


<a id="sec-13-2"></a>
### 13.2 Mandatory Methods and Status Codes

Methods and codes below are REQUIRED for interoperability.

**Methods**
- `GET` — idempotent fetch of resource `u`; body MUST be empty.
- `PUT` — store body at resource `u`; creates or replaces.
- `POST` — create subordinate of `u`; server returns location in `Location` header.
- `DEL` — delete resource `u`.

**Status codes**
- 200 OK, 201 Created, 204 No Content
- 400 Bad Request, 401 Unauthorized, 403 Forbidden, 404 Not Found, 409 Conflict
- 429 Too Many Requests
- 500 Internal Error, 503 Unavailable

Headers (`h`) use lower‑case ASCII keys. Servers MUST preserve unknown headers.

<a id="sec-13-3"></a>
### 13.3 Streaming and Backpressure

**Streaming.** Bodies MAY be streamed across multiple L2 `STREAM` frames. The sender sets FIN on the final frame.

**Backpressure.** Receivers advertise credit via `WINDOW_UPDATE`; senders MUST NOT exceed advertised credit. If credit is zero for 15s, senders MAY close the stream with status 503.

**Order and reassembly.** In‑order delivery per StreamID is required; receivers MAY buffer gaps up to 64 KB per stream.

## 14. Translation Layer for v1.1 Interoperability

<a id="sec-14-1"></a>
### 14.1 Overview

The Translation Layer (TL) allows a v1.2 Node to interoperate with a v1.1 peer without sacrificing L2 security. TL operates entirely at L2 and L3 and never exposes plaintext beyond the v1.2 Node boundary.

<a id="sec-14-2"></a>
### 14.2 Discovery

If a peer offers "betanet/htx/1.1.x" in capabilities, the Decider MAY select it only if TL is enabled. Otherwise negotiation fails.

<a id="sec-14-3"></a>
### 14.3 Mapping

L2 frames 1.1 to 1.2:
- STREAM: identical semantics; map headers to 1.2 structure.
- WINDOW_UPDATE: convert window units 1:1.
- KEY_UPDATE: if absent in 1.1, TL MUST synthesize periodic 1.2 KEY_UPDATE every 64 MiB or 15 minutes, whichever comes first.
- CLOSE: pass-through.

L1 path headers:
- If 1.1 lacks Timestamp, TL MUST synthesize Timestamp from the received packet arrival time and set Expiry accordingly; the 1.2 node MUST re-sign the segment for onward forwarding.

Version binding:
- TL MUST include "compat=1.1" in exporter context to ensure keys differ from pure 1.2 sessions.

<a id="sec-14-4"></a>
### 14.4 Security

TL MUST prevent downgrade confusion by refusing mixed-mode where a peer simultaneously advertises 1.1 and 1.2 but oscillates selection. TL MUST log a signed transcript hash for audits.

<a id="sec-15"></a>
## 15. Error Handling

Codes (u16):
- 0x0000 OK
- 0x0001 VERSION_NEGOTIATION_FAILED
- 0x0002 AUTH_FAILED
- 0x0003 PATH_INVALID
- 0x0004 REPLAY
- 0x0005 FLOW_CONTROL_VIOLATION
- 0x0006 OLD_KEY_FRAME
- 0x0007 MALFORMED_FRAME
- 0x0008 INTERNAL_ERROR
- 0x0009 POLICY_REJECTED
- 0x000A UNSUPPORTED_FEATURE

On error, peers SHOULD send CLOSE with Code and a human-readable Reason. Section 16 centralizes the canonical registry for error codes.

<a id="sec-16"></a>
## 16. Registries (Canonical)

<a id="sec-16-1"></a>
### 16.1 Registry Envelope and Hashing

This document establishes registries maintained in a decentralized manner. Each registry entry is a DET-CBOR record signed by at least 5 maintainers (threshold). Conflicting entries are resolved by lexicographically highest multihash of the record. Maintainer admission and removal are recorded in a Maintainers registry and require signatures from at least two thirds of current maintainers.

Registry Signature Envelope
- Record: DET-CBOR payload
- Signatures: array of Ed25519 signatures over SHA-256("BN-REG" || Record)
- Threshold: 5 minimum, and at least two thirds of listed maintainers

Registry Record Hashing
- Hash = multihash(sha2-256, DET-CBOR bytes)

<a id="sec-16-2"></a>
### 16.2 Established Registries

- L2 Frame Type Registry (0x00 to 0x7F unassigned)
  - 0x00 STREAM
  - 0x01 WINDOW_UPDATE
  - 0x02 PING
  - 0x03 KEY_UPDATE
  - 0x04 CLOSE

- L3 Capability Strings Registry
  - "betanet/htx/1.2.0"
  - "betanet/mesh/1.2.0"
  - "betanet/mix/1.2.0"
  - "betanet/naming/1.2.0"

- L5 Alias Signers Registry
  - Bootstrap signer records and rotations.

- Error Codes Registry
  - 0x0000 through 0x7FFF: Standards Track
  - 0x8000 through 0xFFFF: Private use

- Front Origin Registry
  - Entry type: "FrontOriginRegistry" with TemplateID, HostPattern, ComplianceProfile, Expiry.

<a id="sec-17"></a>
## 17. Security Considerations

- Binding: Version choices and TemplateID are bound into the L2 key schedule to prevent reflection and downgrade attacks.
- Replay: Nonces derive from per-direction counters; peers MUST NOT reuse counters across keys and MUST NOT permit counter wrap.
- Traffic analysis: Cover behavior MUST emulate origin templates per Section 8.5.
- Compromise: Loss of a Node static key does not compromise past sessions due to forward secrecy.
- Privacy hops: Relays SHOULD be jurisdictionally and topologically diverse; clients SHOULD rotate selection every 24 hours.

<a id="sec-18"></a>

## 18. Operational Considerations

**Time synchronization.** NTP or equivalent is REQUIRED.

**NAT traversal and keep-alives.**
- QUIC: send an ACK-eliciting packet or PING at least every 20 seconds when idle; QUIC idle timeout MUST be ≥ 45 seconds.
- TLS/TCP: send an HTTP/2 PING frame at least every 30 seconds when idle.
- Peers MUST send `GOAWAY`/`CONNECTION_CLOSE` before shutdown and wait 2 RTTs for draining.

**Retry/backoff.**
- Connection attempts use exponential backoff with full jitter: initial 1s, factor 2.0, cap 2 minutes.
- Per-peer consecutive failure cap: 5 attempts before a 10‑minute cool‑down.

**PMTU and sizing.**
- QUIC Initial path MUST keep `UDP datagram ≤ 1200` bytes; implementations MUST ensure that L2 frame payloads on initial paths fit within this bound.
- After validation, peers MAY raise L2 frame sizes up to 16 KB, discovering PMTU via QUIC PMTUD (RFC 9000 §14). Blackhole detection MUST revert to 1200‑byte safe size.

**Congestion control interaction.**
- The outer transport (TCP/QUIC) controls congestion. L2 flow control windows MUST be configured such that `L2_window ≥ 2 × cwnd_estimate`. If `cwnd` is unknown, set `L2_window ≥ 256 KB`.
- Senders MUST avoid burst sends larger than the peer’s current `cwnd`; chunk STREAM frames accordingly.

**Connection draining.**
- On transient path failure, buffer for at most 2 seconds; beyond that, send `PATH_CONTROL/ERROR=PEER_DOWN` and tear down the stream.

**Logging.**
- Implementations MUST support privacy-preserving logs that store only transcript hashes and error codes; no payloads.

These parameters are normative and directly affect wire-visible behavior; deviations render a node non‑compliant.
## 19. Compliance Profiles

- MINIMAL: L1, L2 (TLS), L3, L7.
- STANDARD: MINIMAL plus L2 (QUIC), L4, L5 plus Section 8.5 compliance.
- EXTENDED: STANDARD plus L6.

<a id="sec-20"></a>

## 20. Test Procedures

Implementations MUST provide tests that cover:

**Crypto and framing**
- L2 AEAD round‑trips with nonces 0..1024 and random AAD.
- KEY_UPDATE handover with 3‑frame overlap and concurrent updates.
- Version binding mismatch causes decryption failure.
- Frame sizing: reject frames where `(Len + 3) > 65535`.

**Layering and mapping**
- Assert that no stand‑alone L1 headers appear on the wire; capture shows only HTTP/2 or HTTP/3 over TLS/QUIC.

**Discovery and bootstrap**
- Deterministic join using 5 seeds; verify gossip expirations, backoff, and admission limits.

**Routing and forwarding**
- Path construction with loop detection; ECMP tie‑break; liveness probes and eviction after 3 timeouts.
- CF handling reduces selection probability through congested hops.

**Relays**
- Delay distribution U[30ms,150ms]; cumulative cap 600ms; error propagation timing.

**Front Origin Registry**
- Signature verification; expiry handling; behavior on TemplateID mismatch per profile.

**BAR mandatory methods**
- GET/PUT respond with 2xx/4xx/5xx as specified; streaming semantics (13.3) including backpressure and FIN handling.

**BN-Ticket header**
- Enforce 256‑byte maximum; base64url without padding; exact 120‑byte decoded length; malformed header closes connection.

Conformance requires all tests to pass.
## 21. References

- RFC2119 Key words for use in RFCs to Indicate Requirement Levels.
- RFC8174 Ambiguity of Uppercase vs Lowercase in RFC 2119.
- RFC8949 Concise Binary Object Representation (CBOR).
- RFC8446 The Transport Layer Security (TLS) Protocol Version 1.3.
- RFC9000 QUIC: A UDP-Based Multiplexed and Secure Transport.
- RFC7748 Elliptic Curves for Security.
- RFC8032 Edwards-Curve Digital Signature Algorithm (EdDSA).

<a id="app-a"></a>
## Appendix A. ABNF and Data Schemas

Ticket-Header = "BN-Ticket:" OWS "v1" ";" OWS params  
params = param *( OWS ";" OWS param )  
param  = token "=" ( token / quoted-string )

<a id="app-b"></a>
## Appendix B. State Machines

L2 Connection (sender side)

```
+-------+   start   +-----------+   outer TLS ok   +---------+
| Idle  | --------> | Calibrate | ---------------> | Handshk |
+-------+           +-----------+                  +----+----+
                                                           |
                                                    inner ok|
                                                           v
                                                       +---+---+
                                                       |  Open |
                                                       +---+---+
                                                           |
                                           key_update (any)|
                                                           v
                                                       +---+---+
                                                       |Pending|
                                                       +---+---+
                                                           |
                                     3 frames ok (new key) |
                                                           v
                                                       +---+---+
                                                       | Open' |
                                                       +---+---+
                                                           |
                                                         close
                                                           v
                                                       +-------+
                                                       | Drain |
                                                       +-------+
```

<a id="app-c"></a>
## Appendix C. Example Encodings and Test Vectors

C.1 HKDF Context and Keys (Example)

EKM (32 bytes):
8b1a9953c4611296a827abf8c47804d7
7f02b27a3b2e5c5ed1fba6b9b5d80752

K0 = HKDF-Extract(salt=0^32, EKM):
89245712909f2d1041d3f26ab06092c2
64df10429ce351e8273ef873dd0cf06e

TrafficSecret_c:
114572d3dfdf0e268fc1547659fde69b
d1f21c59e5655cc182f78dd8b6f8a7fb

Key_c (send key):
743dc35fbdfe9f893161f6272858a4c4
558a9faddc9a3980218d05367ea6b7bf

NonceSalt_c:
ec8decc0494bc6b8acfc0dd5

C.2 L2 STREAM Frame (Example)

Inputs:
Type=0x00, StreamID=3, AAD=0x0000  
Flags=0x00, Offset=0, Data="Hello, Betanet 1.2!"  
Counter=0

Nonce:
ec8decc0494bc6b8acfc0dd5

Len (u24, excludes the 3-byte Len field):
000035

AAD bytes (u24 Len || Type || StreamID || u16 AAD):
00003500000000030000

Frame bytes (hex):
0000350000000003000026d3fd26498a
080a4e63b0c892e5fecdd95bcb6ea4d1
148d1c49c66eca925c4805d0d8854469
f948e2b08d56e4a3

Note: The frame MUST fit in one L1 packet; L1 Payload Len = 3 + Len.

C.3 L1 Path Segment (Example)

Fixed Timestamp (ms since epoch): 1755172800000

Segment fields:
- NextNode: SHA-256("next-node")[:20]
- Expiry: Timestamp + 300000
- Timestamp: as above
- SigLen: 64
- Sig: Ed25519 over SHA-256("BN-L1-seg" || Version || Type || Flags || NextNode || Expiry || PrevNode || Timestamp)

<a id="app-d"></a>
## Appendix D. Pseudocode Routines

Deterministic CBOR canonical encoding function referenced in Section 6.2.

```
encode(V):
  if V is unsigned integer: return cbor_uint(V)   # shortest form
  if V is negative integer: return cbor_nint(V)   # shortest form
  if V is bytes:            return cbor_bstr(V)   # definite length
  if V is utf8 string:      return cbor_tstr(V)   # definite length; reject invalid UTF-8
  if V is array:
     out = []
     for item in V: out.append(encode(item))
     return cbor_array(len(V)) || concat(out)
  if V is map:
     pairs = []
     for (k,v) in V.items():
        ek = encode(k)
        ev = encode(v)
        pairs.append((ek, ev))
     pairs.sort(key=lambda p: p[0])
     return cbor_map(len(pairs)) || concat(ek||ev for (ek,ev) in pairs)
  else: REJECT
```

HKDF helpers

```
hkdf_expand_label(secret, label, L):
  return HKDF-Expand(secret, "betanet " || label, L)

next_keys(ts):
  ts_prime = hkdf_expand_label(ts, "next", 32)
  key  = hkdf_expand_label(ts_prime, "key", 32)
  nsl  = hkdf_expand_label(ts_prime, "nonce", 12)
  return ts_prime, key, nsl
```