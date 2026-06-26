// client can request a forward stream from relay for a src to a dst, in the future
// the src will be required to also sign and agree to this, as this may be security vulnerability,
// for simplicity, this will accept any

use super::*;

pub enum Client<A = UnsetAlgorithm, B = UnsetDns, C = UnsetProtocol> {
	Outbound {
		secret_key: lib_cryptography::secret_key::SecretKey<A>,
		dns: B,
		route: Route<C>
	},
	Pending {
		attempt: std::time::Instant
	},
	ConnectionRejection,
	Connection,
	TimedOut,
	Illegal
}

impl<A, B, C> From<(lib_cryptography::secret_key::SecretKey<A>, Route<C>)> for Client<A, B, C> 
where
	B: Default {
	fn from(value: (lib_cryptography::secret_key::SecretKey<A>, Route<C>)) -> Self {
		let (secret_key, route) = value;
		Self::Outbound {
			secret_key,
			dns: B::default(),
			route
		}
	}
}

impl<A, B, C> Workflow for Client<A, B, C> 
where 
	A: lib_cryptography::AsymmetricSetLayout,
	A: lib_cryptography::AsymmetricKeyDerivationAlgorithm,
	A: lib_cryptography::AsymmetricSignatureAlgorithm,
	C: Clone,
	C: Send {
	fn next(
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self {
		match self {
			Self::Outbound {
				secret_key,
				dns,
				route
			} => {
				let peer_id: &libp2p::PeerId = swarm.local_peer_id();
				let peer_id: libp2p::PeerId = peer_id.to_owned();
				
				let packet: lib_packet::Unsigned<Route<C>, C> = lib_packet::Unsigned::from_payload(route);
				let packet: lib_packet::MarkedSignedVerified<Route<C>, A, C> = (packet, &secret_key).try_into().unwrap();
				let packet: lib_bytes::NonEmpty = packet.try_into().unwrap();
				let packet: bytes::Bytes = packet.into();
				let packet: sub_system::stream::Packet<C> = (peer_id, packet).into();
				let packet: sub_system::stream::Outbound<_> = packet.into();
				
				queue(Event::from_any(packet));
				
				Self::Pending {
					attempt: std::time::Instant::now()
				}
			},
			Self::Pending {
				attempt
			} => {
				// catch inbound bytes and try to break out of pending
				let Some(_) = event.downcast_ref() else {
					return self
				};
				
				if attempt.elapsed() > std::time::Duration::from_mins(1) {
					
				}
			},
			_ => self
		}
	}
}

impl<A, B, C> Termination for Client<A, B, C> {
	fn is_ready_to_unmount(&self) -> bool {
		match self {
			Self::Illegal => true,
			Self::TimedOut => true,
			_ => false
		}
    }
}