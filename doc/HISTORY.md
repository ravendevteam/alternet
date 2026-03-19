# Project History

This document provides historical and scope clarity for contributors and reviewers, and does not constitute a design document or roadmap. It is intended as a static reference and not as an ongoing discussion or commentary.

## Alternet

Alternet is an experimental, censorship-resistant web protocol under active development as a research and engineering project.

The current scope of Alternet is intentionally limited to:
- the development of a formal Alternet 1.0 protocol specification, and
- a proof-of-concept (PoC) implementation that exercises that specification.

Alternet is not a production-ready network. It does not claim to replace the physical Internet, and it does not claim properties beyond those explicitly stated and demonstrated in its specification and reference implementation.

Technical development is led by a designated lead developer. Development is focused on correctness, explicit threat modeling, clearly stated assumptions, and falsifiable claims.

## Historical Background (Betanet)

Alternet originated from an earlier project phase and name, Betanet.

Early Betanet materials consisted of a conceptual outline exploring approaches to censorship resistance, peer-to-peer communication, and application-layer protocols. These materials were exploratory in nature and did not constitute a complete or stable protocol specification.

Several early decisions contributed to confusion regarding maturity and readiness:
- Draft documents were labeled as "1.0" despite being preliminary outlines.
- Developer bounties were released before a stable and reviewable specification existed.
- Public-facing language used informal terminology (for example, "internet") where more precise technical language (for example, "web" or "application layer") would have been appropriate in a technical context.

These factors collectively created incorrect expectations regarding scope, stability, and implementation readiness.

## Independent Development Efforts During Betanet

During the Betanet phase, multiple independent external teams began experimental implementation work in response to publicly posted bounties.

These teams:
- were not members of the Raven core development team,
- operated independently and without coordination, and
- did not have authority over protocol design, specifications, or project direction.

Due to the absence of a stable specification, unclear boundaries around ownership, and premature incentive structures, these efforts encountered conflicts and were discontinued.

The discontinuation of these external efforts did not represent abandonment of the core project. It reflected the failure of an early bounty structure that preceded sufficient specification maturity.

## Acknowledged Corrections and Project Reset

Following the Betanet phase, the following corrections were explicitly made:

- Project leadership responsibilities were restructured.
- Technical authority over protocol design was delegated to a dedicated lead developer with professional experience in systems and networking.
- Project scope was reduced to a formal specification and a proof-of-concept implementation.
- The project was renamed to Alternet to reflect a reset in expectations, scope, and direction.
- Early drafts, repositories, and documents from the Betanet phase were deprecated.

The original project initiator remains involved in a managerial and oversight capacity but does not serve as the technical authority for protocol design or implementation.

## Funding and Non-Goals

Alternet is not a cryptocurrency project.

No token, coin, or on-chain governance mechanism is part of Alternet's design. Any third-party fundraising or tokenization efforts that occurred outside the core development process did not define project governance, architecture, or direction.

Alternet explicitly does not aim to:
- launch a production network,
- replace the Internet's physical infrastructure,
- provide anonymity guarantees beyond those defined in the specification, or
- make claims that cannot be evaluated through documentation and code.

## How to Evaluate Alternet

Alternet should be evaluated solely on:
1. the clarity, precision, and internal consistency of the Alternet 1.0 specification, and
2. the behavior, limitations, and assumptions demonstrated by the proof-of-concept implementation.

Past project phases are documented here for context only and are not indicative of current architecture, goals, or maturity.