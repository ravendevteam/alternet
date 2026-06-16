use super::*;

// represents the flow of a request for asking relay to
// forward x to anther peer
// including 

#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub enum Reservation<T> {
	Raw(bytes::Bytes),
	Request {
		key: String,
		src: libp2p::Multiaddr,
		dst: libp2p::Multiaddr,
		dns: T
	},
	Validation,
	ProofRequest,
	Renewal,
	Expiration,
	Proof,
	Invalid
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
			Self::Raw(bytes) => {
				let content: Vec<_> = bytes.to_vec();
				let content: identity::SignedUnverified = content.try_into().unwrap();
				let content: identity::SignedVerified = content.try_into().unwrap();
				let (pk, mg) = content.unpack();
				
				// verified and all good
				
				// unpack src and dst payload data
				let src: &str = segments.get(2).unwrap();
				let src: libp2p::Multiaddr = src.parse().unwrap();
				let dst: &str = segments.get(3).unwrap();
				let dst: libp2p::Multiaddr = dst.parse().unwrap();
				
				// state transition
				Self::Request {
					key: nanoid::nanoid!(),
					src,
					dst,
					dns: T::default()
				}
			},
			Self::Request {
				key,
				src,
				dst,
				dns
			} => {
				tokio::runtime::Handle::current().block_on(async move {
					dns.account_has_sufficient_balance(account).await.ok();
					
				});
				
				self
			},
			Self::Validation => {
				
			},
			Self::ProofRequest => {
				
			},
			_ => self
		}
	}
}