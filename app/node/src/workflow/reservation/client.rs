// client can request a forward stream from relay for a src to a dst, in the future
// the src will be required to also sign and agree to this, as this may be security vulnerability,
// for simplicity, this will accept any

use super::*;

#[derive(Clone)]
pub enum Client<A = UnsetAlgorithm, B = UnsetDns, C = UnsetProtocol> {
	Outbound {
		secret_key: lib_cryptography::secret_key::SecretKey<A>,
		dns: B,
		reserver: Identity<A>,
		route: Route<C>
	},
	Pending {
		reserver: Identity<A>,
		attempt: std::time::Instant
	},
	Ongoing,
	TimedOut,
	Illegal
}

impl<A, B, C> From<(lib_cryptography::secret_key::SecretKey<A>, Identity<A>, Route<C>)> for Client<A, B, C> 
where
	B: Default {
	fn from(value: (lib_cryptography::secret_key::SecretKey<A>, Identity<A>, Route<C>)) -> Self {
		let (secret_key, reserver, route) = value;
		Self::Outbound {
			secret_key,
			dns: B::default(),
			reserver,
			route
		}
	}
}

impl<A, B, C> Workflow for Client<A, B, C> 
where 
	A: Clone,
	A: PartialEq,
	A: lib_cryptography::AsymmetricSetLayout,
	A: lib_cryptography::AsymmetricKeyDerivationAlgorithm,
	A: lib_cryptography::AsymmetricSignatureAlgorithm,
	C: 'static,
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
				reserver,
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
					reserver,
					attempt: std::time::Instant::now()
				}
			},
			Self::Pending {
				ref reserver,
				attempt
			} => {
				// catch inbound bytes and try to break out of pending
				if let Some(sub_system::stream::Inbound::<C>(sub_system::stream::Packet {
					peer,
					content,
					..
				})) = event.downcast_ref() {
					let (reserver, reserver_peer) = &reserver.to_owned().into();

					if peer == reserver_peer {
						let packet: lib_bytes::NonEmpty = content.to_owned().try_into().unwrap();
						let packet: lib_packet::MarkedSignedVerified<Status<C>, A, C> = packet.try_into().unwrap();	
						
						if packet.signer() == reserver && packet.code == 200 {
							log::info!("connection established, {} will now forward packets", reserver_peer.to_owned());
							
							return Self::Ongoing
						}
					}
				};
				
				if attempt.elapsed() > std::time::Duration::from_mins(1) {
					Self::TimedOut
				} else {
					self
				}
			},
			Self::Ongoing => self,
			_ => self
		}
	}
	
	fn is_ready_to_unmount(&self) -> bool {
		match self {
			Self::Illegal => true,
			Self::TimedOut => true,
			_ => false
		}	
	}
}