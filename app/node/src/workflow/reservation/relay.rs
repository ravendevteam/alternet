use super::*;

#[derive(Clone)]
pub enum Relay<A = UnsetAlgorithm, B = UnsetDns, C = UnsetProtocol> {
	Inbound {
		caller: Identity<A>,
		route: Route<C>,
		dns: B
	},
	Ongoing
}

impl<A, B, C> FromContext for Relay<A, B, C> 
where
	A: Clone,
	A: PartialEq,
	A: lib_cryptography::AsymmetricSetLayout,
	A: lib_cryptography::AsymmetricKeyDerivationAlgorithm,
	A: lib_cryptography::AsymmetricSignatureAlgorithm,
	B: Default,
	B: Dns<ForeignAlgorithm = A>,
	C: 'static,
	C: Clone,
	C: Send {
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
		let packet: lib_bytes::NonEmpty = content.to_owned().try_into().unwrap();
		let packet: lib_packet::MarkedSignedVerified<Route<C>, A, C> = packet.try_into().unwrap();
		let dns: B = B::default();
		
		// in the future, this can be make fully async to stop it from blocking the entire program
		tokio::runtime::Handle::current().block_on(async move {
			let signer: &lib_cryptography::public_key::PublicKey<_> = packet.signer();
			let signer: lib_cryptography::public_key::PublicKey<_> = signer.to_owned();
			
			// verification
			
			let caller = Identity::from((signer, peer.to_owned()));

			// expanded in cryptography focused milestone: here we bind the reservation cryptographically
			// here we would generate a shared commitment and expand the handshake

			Vec::from([
				Self::Inbound {
					caller,
					dns,
					route: packet.content().to_owned()
				}
			])
		})
	}
}

impl<A, B, C> Workflow for Relay<A, B, C>
where
	B: Dns<ForeignAlgorithm = A>,
	B: Default,
	C: 'static,
	C: Send {
	fn next(
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self {
		match self {
			Self::Inbound {
				caller,
				route,
				dns
			} => {
				let src: libp2p::PeerId = route.src;
				let src: sub_system::stream::Peer<C> = src.into();
				let dst: libp2p::PeerId = route.dst;
				let dst: sub_system::stream::Peer<C> = dst.into();
				let route: sub_system::forward::Route<_> = sub_system::forward::Route {
					src,
					dst
				};
				let event: sub_system::forward::Insert<_> = route.into();
				let event: Event = Event::from_any(event);
				
				queue(event);
				
				// needs to have another state where it receives feedback from the forwarding sub system
				
				Self::Ongoing
			},
			Self::Ongoing => {
				// here there would be a timeout phase for or a renewal signal from the client
				// this should also be where we catch forward packets for signatures
				
				self
			}
		}
	}
}