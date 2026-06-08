use super::*;

pub struct Packet {
	src: PublicKey,
	src_sig: Signature,
	dst: PublicKey,
	// relays it will pass through and they have said its ok
	stops: Vec<(PublicKey, Signature)>
}

pub enum Handshake {
	Request {
		src: PublicKey,
		src_sig: Signature,
		dst: PublicKey
	},
	Connect {
		
	},
	Success {
		
	},
	Failure
}

impl SubSystem for Handshake {
	fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
		let Some(SwarmEvent::Behaviour())
	}
}