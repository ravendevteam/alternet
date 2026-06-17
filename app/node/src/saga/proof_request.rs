use super::*;

pub enum ProofRequest {
	Inbound(Vec<u8>),
	Sign {
		
	},
	Outbound(Vec<u8>)
}

impl Saga for ProofRequest {
	fn next(
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self {
		// await proof
	}
}