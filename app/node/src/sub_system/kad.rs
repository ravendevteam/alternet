use super::*;

pub struct Query {
	pub key: bytes::Bytes,
	pub sx: tokio::sync::mpsc::Sender<bytes::Bytes>
}

pub struct Kad {
	pending: std::collections::HashMap<libp2p::kad::QueryId, tokio::sync::mpsc::Sender<bytes::Bytes>>
}

impl SubSystem for Kad {
	fn receive(
		&mut self, 
		swarm: &mut Swarm, 
		event: &mut Event, 
		queue: &mut dyn FnMut(Event)
	) {
		if let Some(Query {
			key,
			sx
		}) = event.downcast_ref() {
			let key = libp2p::kad::RecordKey::new(&key);
			let id = swarm.behaviour_mut().kad.get_record(key);
			
			self.pending.insert(id, sx.to_owned());
		}
		
		if let Some(SwarmEvent::Behaviour(BehaviourEvent::Kad(libp2p::kad::Event::OutboundQueryProgressed {
			id,
			result: libp2p::kad::QueryResult::GetRecord(Ok(libp2p::kad::GetRecordOk::FoundRecord(libp2p::kad::PeerRecord {
				peer,
				record
			}))),
			stats,
			step
		}))) = event.downcast_ref() {
			if let Some(sx) = self.pending.remove(id) {
				let bytes: bytes::Bytes = bytes::Bytes::copy_from_slice(&record.value);
				
				tokio::spawn(async move {
					sx.send(bytes).await.ok();
				});
			}
		}
	}
}