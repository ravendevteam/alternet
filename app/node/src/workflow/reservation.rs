use super::*;

pub mod client;
pub mod relay;

pub struct UnsetProtocol;
pub struct UnsetAlgorithm;
pub struct UnsetDns;

#[derive(Debug)]
#[derive(Clone)]
struct Route<T = UnsetProtocol> {
	phantom_data: std::marker::PhantomData<T>,
	src: libp2p::PeerId,
	dst: libp2p::PeerId
}

impl<T> TryFrom<lib_bytes::NonEmpty> for Route<T> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: lib_bytes::NonEmpty) -> std::result::Result<Self, Self::Error> {
		let bytes: bytes::Bytes = value.into();
		let bytes: Vec<_> = bytes.to_vec();
		
		let buffer: String = String::from_utf8(bytes)?;
		
		let mut segments: std::str::SplitWhitespace = buffer.split_whitespace();
		let src: &str = segments.next().ok_or("missing source peer id")?;
		let src: libp2p::PeerId = src.parse()?;
		let dst: &str = segments.next().ok_or("missing destination peer id")?;
		let dst: libp2p::PeerId = dst.parse()?;
		let out: Self = Self {
			phantom_data: std::marker::PhantomData,
			src,
			dst
		};
		Ok(out)
	}
}

impl<T> TryInto<lib_bytes::NonEmpty> for Route<T> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_into(self) -> std::result::Result<lib_bytes::NonEmpty, Self::Error> {
		let src: String = self.src.to_string();
		let dst: String = self.dst.to_string();
		
		let mut buffer: String = String::default();
		buffer.push_str(&src);
		buffer.push_str(" ");
		buffer.push_str(&dst);
		buffer.push_str(" ");
		
		let bytes: Vec<_> = buffer.into_bytes();
		let bytes: bytes::Bytes = bytes.into();
		let bytes: lib_bytes::NonEmpty = bytes.try_into()?;
		
		Ok(bytes)
	}
}

struct Out {
	secret_key: lib_cryptography::secret_key::SecretKey<>
}

struct Inbound(lib_bytes::NonEmpty);

struct Validation<A = UnsetAlgorithm, B = UnsetDns> {
	key: String,
	signer: lib_cryptography::public_key::PublicKey<A>,
	signature: lib_cryptography::signature::Signature<A>,
	request: Route,
	dns: B
}




pub enum Relay {
	
}

pub enum Reservation<A = UnsetAlgorithm, B = UnsetDns, C = UnsetProtocol> {
	Outbound(Out),
	Inbound(Inbound),
	Validation(Validation<A, B>),
	Connection,
	Illegal
}

impl<A, B, C> From<(libp2p::PeerId, libp2p::PeerId)> for Reservation<A, B, C> {
	fn from(value: (libp2p::PeerId, libp2p::PeerId)) -> Self {
		let (src, dst) = value;
		
		let src = src.to_bytes();
		let dst = dst.to_bytes();
		
		// merge into one unique bytes stream -- future me 
		
		Self::Outbound(src)
	}
}

impl<A, B, C> Workflow for Reservation<A, B, C>
where
	A: lib_cryptography::AsymmetricSignatureAlgorithm,
	B: Default,
	B: Dns<Algorithm = A> {	
	fn next(
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self {
		match self {
			// from client to relay
			Self::Outbound(Out {
				secret_key
			}) => {
				// dispatch outbound bytes to the target relay
				// 
				
				let peer_id = ;
				let event: lib_packet::Unsigned<_, _> = bytes.try_into().unwrap();
				let event: lib_packet::MarkedSignedVerified<_, _, _> = (event, secret_key).try_into().unwrap();
				let event: lib_bytes::NonEmpty = event.try_into().unwrap();
				let pkt: sub_system::stream::Packet<_> = (peer_id, event).into();
				let pkt: sub_system::stream::Outbound<_> = pkt.into();
				let pkt = Event::from_any(pkt);
				queue(pkt);
				
				Self::Inbound(_)
			},
			// receive as relay
			Self::Inbound(Inbound(bytes)) => {
				let content: bytes::Bytes = bytes.into();
				let content: Vec<_> = content.to_vec();
				let content: bytes::Bytes = content.into();
				let content: lib_bytes::NonEmpty = content.try_into().unwrap();
				let content: lib_packet::MarkedSignedUnverified<Route, A, C> = content.try_into().unwrap();
				let content: lib_packet::MarkedSignedVerified<Route, A, C> = content.try_into().unwrap();
				let (request, signer, signature) = content.into();
				Self::Validation(Validation {
					key: nanoid::nanoid!(),
					signer,
					signature,
					request,
					dns: B::default()
				})
			},
			Self::Validation(Validation {
				key,
				signer,
				signature,
				request,
				dns
			}) => {
				tokio::runtime::Handle::current().block_on(async move {
					let Ok(true) = dns.account_has_sufficient_balance(signer).await else {
						return Self::Illegal
					};
					
					// expanded opon in cryptography focused milestone: here we bind the reservation cryptographically
					dns.accept_commitment().await;
					
					Self::Connection
				})
			},
			Self::Connection => {
				// establish an ongoiung forwarded stream to the requested destination
				Self::Illegal
			},
			_ => self
		}
	}
}

impl<A, B, C> Termination for Reservation<A, B, C> {
	fn is_ready_to_unmount(&self) -> bool {
		match self {
			Self::Illegal => true,
			_ => false
		}
    }
}