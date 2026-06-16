use super::*;

#[derive(Debug)]
pub struct Route<T> {
	pub src: stream::Peer<T>,
	pub dst: stream::Peer<T>
}

#[derive(Debug)]
#[derive(derive_more::From)]
pub struct Insert<T>(Route<T>);

#[derive(Debug)]
#[derive(derive_more::From)]
pub struct Remove<T>(Route<T>);

#[derive(Debug)]
pub struct Forward<T> {
	phantom_data: std::marker::PhantomData<T>,
	
	// currently no maximum capacity per src, this will need to be fixed for stability purposes in the future
	src_to_dsts: std::collections::HashMap<libp2p::PeerId, Vec<libp2p::PeerId>>
}

impl<T> Default for Forward<T> {
	fn default() -> Self {
		Self {
			phantom_data: std::marker::PhantomData,
			src_to_dsts: std::collections::HashMap::default()
		}
	}
}

impl<T> SubSystem for Forward<T>
where
	T: 'static,
	T: Send,
	T: stream::Protocol {
	fn receive(
		&mut self, 
		swarm: &mut Swarm, 
		event: &mut Event, 
		queue: &mut dyn FnMut(Event)
	) {
		if let Some(stream::Inbound::<T>(stream::Packet {
			peer,
			content,
			..
		})) = event.downcast_ref() {
			let peer = peer.to_owned();
			let dsts = self.src_to_dsts.entry(peer).or_default();
			if !dsts.is_empty() {
				for dst in dsts {
					// parrot the packet to the destination
					// stream sub system should see this and set up a new stream if there is none
					queue(Event::from_any(stream::Outbound::<T>::from(stream::Packet::from((dst.to_owned(), content.to_owned())))));
				}
			}
		}
		
		if let Some(Insert::<T>(Route {
			src,
			dst
		})) = event.downcast_ref() {
			let dsts = self.src_to_dsts.entry((*src).to_owned()).or_default();
			dsts.push((*dst).to_owned());
		}
		
		if let Some(Remove::<T>(Route {
			src,
			dst
		})) = event.downcast_ref() {
			
		}
	}
}