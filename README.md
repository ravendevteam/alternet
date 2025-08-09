# BetaNet Alternative Implementation
This adaptation of `BetaNet` is built on torrenting and quantum computer resistant algorithms. The network will also be constantly be broadcasting client information which will obscure genuine requests to pages.
In a traditional model (i.e. the current internet), a client establishes a direct communication with a server (in the handshake/key exchange process). However, this is already a threat to privacy because anyone on that network can see the handshake. Furthermore, the client server model is flawed in that the server will need to become incredibly powerful as demand grows.
This model aims to solve both these problems.
## How does a peer stay in the network?
A peer will keep track of all peers it has ever encountered. If a serving peer receives a connection from a requesting peer, all peers the serving peer knows, will be sent the details of the connecting peer to add to their list. This means that over time, as connections are sent/received to and from that peer, it's list of known peers will grow (a cap can be put on the size of this list and peers that are offline for long periods of time can be removed, this can be done by the user).
## How will pages be queried?
When a peer wants to request a page, it selects 100 (or any other number) of  peers in it's list to establish a communication with. Once a safe connection has been established (e.g. using 3 way asymmetric encryption), parts of the page are queried (where each part is 512 bytes of the page, or something like that). Eventually all parts are collected to reconstruct the original page. (The parts are also crosschecked with other peers to ensure they have not been tampered with. Chunks can also be queried at the same time.) The constructed page can then be displayed to the user. The user also stores some chunks of the page so it may act as a serving peer in the future.

**NB:** The peers also exchange their lists of peers at this stage, (described in `How does a peer stay in the network?`).
## How will new pages be published?
Ultimately, this depends on the scale of the network. In any case, a page is broken down into parts, the parts of the page are pushed to random peers (each peer stores all the parts of the page). If these peers are well connected (i.e. lots of other peers have that peer stored in their lists), requests can be made immediately afterwards. The hope is that because that website is sent to many peers, those peers are likely to be stored in the list of other peers.
## Updating pages
Unfortunately, there is no good way to update pages. Pages can only be created, they can't be removed or updated. The only reasonable way to do updates is to push the newer version of the page and have the old one fade into irrelevance. 
Pages will be tagged with a version, and that version must be common among the majority of the other known peers, if it is, it is requested and the outdated parts the client has stored are updated.
If an attacker wanted to inject a malicious page into the clients and the network (the attacker will claim to have a higher version than what is currently available, e.g. the real version is 1.2.3 but the attacker's version is 9.9.9, the attacker's page would be treated as the "updated version" and thus be downloaded) they would need to be well connected and control over half of the clients in the network. Which becomes increasingly challenging as the network grows. (e.g. if the network has 1,000,000 active users, the attacker needs to create 1,000,000 clients, (minimum) to stand a change).

## Construction Notice

- This discusses an implementation that doesn't hinge on the client server model (drawbacks discussed at the start of this file).
- A detailed breakdown of the protocol will be produced if this proposal is accepted.
- The encryption schemes proposed will be quantum resistant (as to avoid `SNDL` and forced migration in 2027).
