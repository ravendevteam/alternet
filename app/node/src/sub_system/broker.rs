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
	pool: saga::Pool<saga::reservation::Reservation<A, B, C>>
}

impl<A, B, C> SubSystem for Broker<A, B, C> 
where
	B: Dns<Algorithm = A> {
	fn receive(
		&mut self, 
		swarm: &mut Swarm, 
		event: &mut Event, 
		queue: &mut dyn FnMut(Event)
	) {
		use saga::Saga as _;
		
		self.pool.next(swarm, event, queue);
	}
}