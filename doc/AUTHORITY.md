# Authority

This document defines the authority structure, roles, decision-making model, and fund usage constraints for the Alternet project. It applies to all Alternet specifications, reference implementations, and associated repositories unless explicitly stated otherwise.

## Scope

This document governs:
- project leadership and authority boundaries,
- protocol design and specification control,
- implementation ownership and responsibilities,
- contribution and review processes, and
- management and use of project-controlled resources.

This document does not govern:
- third-party or downstream projects,
- independent implementations, or
- informal community discussion spaces.

## Roles

### Project Manager

The Project Manager oversees the Alternet project at an organizational level. Responsibilities include:
- defining and maintaining the overall project vision,
- coordinating project direction and priorities,
- overseeing public communication and project updates,
- managing Alternet community funds and project resources,
- ensuring that development efforts align with the intended project vision.

The Project Manager does not serve as the technical authority for protocol design or implementation details.

### Technical Lead

The Technical Lead is the final authority on all technical decisions related to Alternet. Responsibilities include:
- protocol architecture and design,
- authorship and revision of the Alternet specification,
- defining technical scope and constraints,
- leading development efforts,
- resolving technical disagreements.

The Technical Lead serves as the protocol architect and directs all technical work. Technical decisions are final.

### Developer

Developers are responsible for implementing Alternet according to the specification defined by the Technical Lead. Responsibilities include:
- developing the proof-of-concept implementation,
- collaborating with the Technical Lead on feasibility and design considerations,
- maintaining implementation correctness and consistency with the specification.

Developers do not hold independent authority over protocol design unless explicitly delegated by the Technical Lead.

## Specification Authority

The Alternet specification is the authoritative definition of the protocol. Implementations exist to:
- validate the specification,
- demonstrate feasibility,
- expose assumptions and limitations.

Implementations do not define protocol behavior independently of the specification.

## Contributions

Contributions are voluntary and may be accepted, modified, or rejected at the discretion of the Technical Lead or delegated maintainers. Participation in development does not create:
- governance authority,
- ownership over specifications,
- entitlement to future decision-making.

Any bounties or incentives, if offered, are limited to explicitly defined deliverables and do not confer governance rights or control.

## Funding Authority

Alternet community funds and project-controlled resources are managed by the Project Manager. Funds may be used only for purposes that have received explicit community authorization as defined in this document. Funds do not represent ownership, equity, or entitlement to governance or technical authority.

### Community Authorization Requests

Use of Alternet community funds requires an approved Community Authorization Request (CAR). A CAR must be submitted by the Project Manager and made available to the community for review and vote prior to any use of funds.

Each CAR must explicitly include:
- the amount requested, denominated in USD,
- the purpose of the request,
- a detailed description of how the funds will be used exclusively,
- a description of what the funds will not be used for,
- an ordered list of actions that will be taken if the request is authorized, and
- reasoning and justification for the request.

Incomplete requests are not eligible for authorization.

Each CAR must be:
- published on the projects public website, and
- brought to public attention through all primary project communication platforms.

Publication must occur prior to the start of the voting period.

Authorization requires a public poll conducted under the following constraints:
- the voting period must remain open for a minimum of 48 hours,
- voting must be limited to one vote per IP address,
- VPN connections may be blocked to enforce vote integrity,
- no additional restrictions may be placed on participation.

The poll results determine whether the request is authorized.

During this time, the Project Manager is responsible for:
- monitoring community feedback during the authorization period,
- collecting feedback through official project communication channels,
- ensuring that feedback is available for review prior to final authorization.

If a CAR is approved, funds may be used only in accordance with the authorized request. Deviation from an approved request requires submission of a new CAR. Failure to obtain authorization invalidates the request regardless of intent or urgency.

## Representative Authority

Only designated project leadership may represent Alternet in an official capacity. Authoritative project information is limited to:
- the Alternet specification,
- official repositories,
- documents published within alternet-docs.

Informal communication channels are not authoritative sources of project decisions.

## Amendments to Outlined Authority

This document may be amended only with review and approval from all individuals holding official roles within the Alternet project at the time of the proposed amendment. The review period begins when the proposed amendment is made available through official project channels.

Proposed amendments must be reviewed in full by each role holder. Approval or rejection must be explicit. If an individual holding an official role does not explicitly approve or reject a proposed amendment within seven (7) days of the amendment being made available for review, their participation is no longer required and the amendment process proceeds without their vote.

All approved amendments must be recorded in version control.