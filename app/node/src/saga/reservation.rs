use super::*;

// represents the flow of a request for asking relay to
// forward x to anther peer
// including 

pub struct IsUnsetProtocol;
pub struct IsUnsetAlgorithm;
pub struct IsUnsetDns;

#[derive(Debug)]
#[derive(Clone)]
struct Request {
	src: libp2p::PeerId,
	dst: libp2p::PeerId
}

impl TryFrom<lib_packet::Unsigned> for Request {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: lib_bytes::NonEmpty) -> std::result::Result<Self, Self::Error> {
		
	}
}

struct Inbound(lib_bytes::NonEmpty);

struct Validation<A = IsUnsetAlgorithm, B = IsUnsetDns> {
	key: String,
	signer: lib_cryptography::public_key::PublicKey<A>,
	signature: lib_cryptography::signature::Signature<A>,
	request: Request,
	dns: B
}

#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub enum Reservation<A = IsUnsetAlgorithm, B = IsUnsetDns, C = IsUnsetProtocol> {
	Inbound(Inbound),
	Validation(Validation<A, B>),
	Connection,
	Illegal
}

impl<A, B, C> Saga for Reservation<A, B, C>
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
			Self::Inbound(Inbound(bytes)) => {
				let content: bytes::Bytes = bytes.into();
				let content: Vec<_> = bytes.to_vec();
				let content: bytes::Bytes = content.into();
				let content: lib_bytes::NonEmpty = content.try_into().unwrap();
				let content: lib_packet::MarkedSignedUnverified<Request, A, C> = content.try_into().unwrap();
				let content: lib_packet::MarkedSignedVerified<Request, A, C> = content.try_into().unwrap();
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