use super::*;

// represents the flow of a request for asking relay to
// forward x to anther peer
// including 

#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub enum Reservation<T> {
	Parse(bytes::Bytes),
	Validation {
		key: String,
		pk: identity::PublicKey,
		src: libp2p::Multiaddr,
		dst: libp2p::Multiaddr,
		ttl: std::time::Instant,
		dns: T
	},
	Connection,
	ConnectionOngoing {
		timeout: std::time::Instant
	},
	ProofRequest {
		signer: identity::PublicKey
	},
	ProofInbound,
	ProofSubmission {
		dns: T
	},
	Renewal,
	Expiration,
	Proof,
	Invalid,
	Complete {
		
		pk: identity::PublicKey,
		// reward received for this reservation
		reward: Balance
	}
}

impl<T> Unique for Reservation<T> {
	fn key(&self) -> &str {
    	""
	}
}

impl<T> Saga for Reservation<T> 
where
	T: Dns {	
	fn next(
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self {
		match self {
			Self::Parse(bytes) => {
				let content: Vec<_> = bytes.to_vec();
				let content: bytes::Bytes = content.into();
				let content: lib_bytes::NonEmpty = content.try_into().unwrap();
				let content: lib_packet::MarkedSignedUnverified<lib_cryptography_algorithm_ed25519::Ed25519Algorithm, sub_system::broker::An> = content.try_into().unwrap();
				let content: lib_packet::MarkedSignedVerified<_, _> = content.try_into().unwrap();
				let (_, public_key, message) = content.into();
				let message: lib_bytes::NonEmpty = message.into();
				let message: bytes::Bytes = message.into();
				let segments: std::iter::Filter<_, _> = message.split(|byte| byte.is_ascii_whitespace()).filter(|chunk| !chunk.is_empty());
				let src: &[u8] = segments.next().unwrap();
				let src: libp2p::Multiaddr = src.parse().unwrap();
				let dst: &[u8] = segments.next().unwrap();
				let dst: libp2p::Multiaddr = dst.parse().unwrap();
				
				
				let src: &str = segments.get(2).unwrap();
				let src: libp2p::Multiaddr = src.parse().unwrap();
				let dst: &str = segments.get(3).unwrap();
				let dst: libp2p::Multiaddr = dst.parse().unwrap();
				let duration: &str = segments.get(4).unwrap();
				let duration: u64 = duration.parse().unwrap();
				let duration: std::time::Duration = std::time::Duration::from_millis(duration);
				let now: std::time::Instant = std::time::Instant::now();
				let ttl: std::time::Instant = now + duration; // how long to hold reservation for, longer means more trust
				
				// state transition
				Self::Validation {
					key: nanoid::nanoid!(),
					pk: public_key,
					src,
					dst,
					ttl,
					dns: T::default()
				}
			},
			Self::Validation {
				key,
				pk: signer,
				src,
				dst,
				dns
			} => {
				tokio::runtime::Handle::current().block_on(async move {
					let Balance(balance) = dns.locked_balance_of(signer.to_owned()).await.unwrap();
					let timeout = dns.locked_balance_timeout_of(signer.to_owned()).await.unwrap();
				});
				
				self
			},
			Self::Connection => {
				let event: sub_system::forward::Route<T> = sub_system::forward::Route {
					src,
					dst
				};
				let event: sub_system::forward::Insert<_> = sub_system::forward::Insert::from(event);
				let event: Event = Event::from_any(event);
				
				queue(event);
				
				Self::ConnectionOngoing
			},
			Self::ConnectionOngoing {
				timeout
			} => {
				let now = std::time::Instant::now();
				
				if now > timeout {
					
				}
				
				// check timer, and how long the stream has been doing for
				// wait for renewal signal or for it to expire
				
			},
			Self::ProofRequest => {
				// send back received content, wait for signer to sign
				
				sub_system::stream::Packet::from();
				sub_system::stream::Outbound::from();
				// generate outbound packet to the client
				
				queue()
			},
			Self::ProofInbound => {
				
			},
			Self::ProofSubmission {
				dns
			} => {
				Proof {
					
				};
				
				tokio::runtime::Handle::current().block_on(async move {
					match dns.receive_proof(proof).await {
						Ok(_) => {
							
						},
						Err(_) => {
							
						}
					}
				});
				
			},
			_ => self
		}
	}
}