use super::*;

pub struct UnsetProtocol;
pub struct UnsetAlgorithm;
pub struct UnsetDns;

pub struct An;

impl stream::Protocol for An {
	fn protocol() -> libp2p::StreamProtocol {
		libp2p::StreamProtocol::new("/an")
	}
}

pub struct Broker<A = UnsetAlgorithm, B = UnsetDns, C = UnsetProtocol> {	
	client_pool: workflow::Pool<workflow::reservation::client::Client<A, B, C>>,
	relay_pool: workflow::Pool<workflow::reservation::relay::Relay<A, B, C>>
}

impl<A, B, C> SubSystem for Broker<A, B, C> 
where
	A: Clone,
	A: PartialEq,
	A: lib_cryptography::AsymmetricSetLayout,
	A: lib_cryptography::AsymmetricKeyDerivationAlgorithm,
	A: lib_cryptography::AsymmetricSignatureAlgorithm,
	A: lib_cryptography::AsymmetricSignatureAlgorithm,
	B: Default,
	B: Dns<Algorithm = A>,
	C: 'static,
	C: Send,
	C: Clone {
	fn receive(
		&mut self, 
		swarm: &mut Swarm, 
		event: &mut Event, 
		queue: &mut dyn FnMut(Event)
	) {
		self.client_pool.next(swarm, event, queue);
		self.relay_pool.generate(swarm, event, queue);
		self.relay_pool.next(swarm, event, queue);
	}
}