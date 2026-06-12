use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct InboundBytes {
	pub src: libp2p::PeerId,
	pub content: bytes::Bytes
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct OutboundBytes {
	pub dst: libp2p::PeerId,
	pub content: bytes::Bytes
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct Blacklist {
	pub peer: libp2p::PeerId
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct Reset {
	pub peer: libp2p::PeerId
}

// blacklist and whitelist peers on the network, bypass routing protocol
// 
// where: router generates events from inbound and outbound packets, firewall,
// screens these packets, and rejects foreign peers
#[derive(Debug)]
pub struct Firewall<T> {
	phantom_data: std::marker::PhantomData<T>,
	peer_to_blacklist: std::collections::HashMap<libp2p::PeerId, bool>,
	
}

impl<T> SubSystem for Firewall<T> 
where
	T: router::Protocol {
	fn receive(
		&mut self, 
		swarm: &mut Swarm, 
		event: &mut Event, 
		queue: &mut dyn FnMut(Event)
	) {
		if let Some(Blacklist {
			peer
		}) = event.downcast_ref() {
			let peer: libp2p::PeerId = peer.to_owned();
			let blacklisted: &mut bool = self.peer_to_blacklist.entry(peer).or_default();
			
			*blacklisted = true;
		}
		
		if let Some(Reset {
			peer
		}) = event.downcast_ref() {
			let peer: libp2p::PeerId = peer.to_owned();
			let blacklisted: &mut bool = self.peer_to_blacklist.entry(peer).or_default();
			
			*blacklisted = false;
		}
		
		// receive inbound bytes from router
    	if let Some(router::InboundBytes::<T> {
     		phantom_data,
     		src,
       		content
     	}) = event.downcast_ref() {
      		
      	}
	}
}