use super::*;

#[derive(Debug)]
pub struct Reject {
	
}

#[derive(Debug)]
pub struct Forward {
	pub src: libp2p::PeerId,
	pub dst: libp2p::PeerId
}

#[derive(Debug)]
pub struct Clear {
	pub src: libp2p::PeerId,
	pub dst: libp2p::PeerId
}

#[derive(Debug)]
pub struct ClearAll {
	pub src: libp2p::PeerId
}

#[derive(Debug)]
pub struct Router {
	src_to_dsts: std::collections::HashMap<libp2p::PeerId, Vec<libp2p::PeerId>>
}

impl SubSystem for Router {
	fn receive(
		&mut self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) {
		
		// mark bytes from a src to be forwarded to an additional dst
		if let Some(Forward {
			src,
			dst
		}) = event.downcast_ref() {
			let dsts: &mut Vec<_> = self.src_to_dsts.entry(Clone::clone(&src)).or_default();
			dsts.push(Clone::clone(&dst));
			dsts.sort();
			dsts.dedup();
		}
		
		if let Some(Clear {
			src,
			dst
		}) = event.downcast_ref() {
			let dsts: &mut Vec<_> = self.src_to_dsts.entry(Clone::clone(&src)).or_default();
			
		}

		if let Some(session_manager::InboundBytes {
			src,
			content
		}) = event.downcast_ref() 
		&& let Some(dsts) = self.src_to_dsts.get(&src) {
			// fan out the forward to multiple other peers
			for dst in dsts {
				let event: Event = Event::new(session_manager::OutboundBytes {
					dst: dst.to_owned(),
					content: content.to_owned()
				});
				
				queue(event);
			}
		}
	}
}
