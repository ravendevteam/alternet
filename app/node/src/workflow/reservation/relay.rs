use super::*;

pub enum Relay<A = UnsetAlgorithm, B = UnsetDns, C = UnsetProtocol> {
	Connection {
		src: libp2p::PeerId,
		dst: libp2p::PeerId,
		signer: lib_cryptography::public_key::PublicKey<A>,
		signature: lib_cryptography::signature::Signature<A>
	},
	Pending,
	Ongoing,
	Illegal
}

impl<A, B, C> FromContext for Relay<A, B, C> {
	fn from_context(
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Vec<Self> {
		let Some(sub_system::stream::Inbound::<C>(sub_system::stream::Packet {
			peer,
			content,
			..
		})) = event.downcast_ref() else {
			return Vec::default()
		};
		let content: lib_bytes::NonEmpty = content.to_owned().try_into().unwrap();
		let content: lib_packet::MarkedSignedUnverified<Route, B, C> = content.try_into().unwrap();
		let content: lib_packet::MarkedSignedVerified<Route, B, C> = content.try_into().unwrap();
		let (content, signer, signature) = content.into();
		let Route {
			src,
			dst
		} = content;
		let dns: B = B::default();

		tokio::runtime::Handle::current().block_on(async move {
			let Ok(true) = dns.account_has_sufficient_balance(signer).await else {
				return Vec::default()
			};

			// expanded in cryptography focused milestone: here we bind the reservation cryptographically
			dns.accept_commitment().await;

			Vec::default()
		})
	}
}

impl<A, B, C> Workflow for Relay<A, B, C>
where
	B: Dns<Algorithm = A>,
	B: Default {
	fn next(
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self {
		match self {
			Self::Connection {
				src,
				dst,
				signer,
				signature
			} => {
				// forward subsystem
				queue()
			},
			_ => self
		}
	}
}

impl<A, B, C> Termination for Relay<A, B, C> {
	fn is_ready_to_unmount(&self) -> bool {
		match self {
			Self::Illegal => true,
			_ => false
		}
	}
}