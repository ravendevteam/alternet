use super::*;

#[derive(Debug)]
#[derive(bon::Builder)]
pub struct PeerRegistry {
	#[builder(into)]
	#[builder(default = std::collections::HashSet::default())]
	peers: std::collections::HashSet<libp2p::PeerId>
}

impl SubSystem for PeerRegistry {
	fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
		if let Some(SwarmEvent::ConnectionEstablished {
			peer_id,
			connection_id,
			endpoint,
			num_established,
			concurrent_dial_errors,
			established_in
		}) = event.downcast_ref() {
			let peer_id: libp2p::PeerId = *peer_id;
			
			self.peers.insert(peer_id);
		}
		
		if let Some(SwarmEvent::ConnectionClosed {
			peer_id,
			connection_id,
			endpoint,
			num_established,
			cause
		}) = event.downcast_ref() {
			if *num_established == 0 {
				self.peers.remove(peer_id);
			}
		}
		
		if let Some(grpc::WhoISee {
			reply
		}) = event.downcast_ref() {
			let peers: Vec<_> = self.peers.iter().copied().collect();
			let peers: Vec<_> = peers.iter().map(|x| x.to_string()).collect();
			let reply: tokio::sync::mpsc::Sender<_> = reply.clone();
			
			tokio::spawn(async move {
				reply.send(peers).await.ok();
			});
		}
	}
}