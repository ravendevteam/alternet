// this needs a redesign, but it should hold up for now
// notable problems are that there is no control on the amount of records
// that we may wait for, which leaves room for a denial of service attack by
// overflowing the record

use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Record {
	goto: libp2p::Multiaddr,
	signature: bytes::Bytes
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
pub struct Query(Domain);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
pub struct Found((Domain, Available));

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
pub struct Available(Vec<Record>);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
pub struct Timeout(std::time::Instant);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
pub struct Entry((Available, Timeout));

// map spaghetti : ( ... needs to be refactored
#[derive(Debug)]
pub struct SearchEngine {
	cache_timeout: std::time::Duration,
	search_to_pending_update: std::collections::HashMap<Query, bool>,
	search_to_query_id: std::collections::HashMap<Query, libp2p::kad::QueryId>,
	search_to_entry: std::collections::HashMap<Query, Entry>,
	query_id_to_search: std::collections::HashMap<libp2p::kad::QueryId, Query>,
	query_id_to_entry: std::collections::HashMap<libp2p::kad::QueryId, Entry>
}

impl SubSystem for SearchEngine {
	fn receive(
		&mut self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) {
		if let Some(search) = event.downcast_ref() {
			if let Some(Entry((available, Timeout(timeout)))) = self.search_to_entry.get(search) {
				let now: std::time::Instant = std::time::Instant::now();

				if &now < timeout {
					let Query(domain) = search.to_owned();
					let event: Found = Found::from((domain, available.to_owned()));
					let event: Event = Event::from_any(event);

					queue(event);

					return
				}
			}

			if self.search_to_pending_update.get(search).is_none() {
				self.search_to_pending_update.insert(search.to_owned(), true);
				self.search_to_entry.remove(search);

				let swarm: &mut Behaviour = swarm.behaviour_mut();
				let Query(Domain(key)) = search;
				let key: &[u8] = key.as_bytes();
				let key: libp2p::kad::RecordKey = libp2p::kad::RecordKey::new(&key);
				let query_id: libp2p::kad::QueryId = swarm.kad.get_record(key);

				self.search_to_query_id.insert(search.to_owned(), query_id);
				self.query_id_to_search.insert(query_id, search.to_owned());
			}
		}

		// eventually refactor to return early as soon as a valid record is found that has been cryptographically mapped to the
		// real owner of the domain
		if let Some(SwarmEvent::Behaviour(BehaviourEvent::Kad(libp2p::kad::Event::OutboundQueryProgressed {
			id,
			result: libp2p::kad::QueryResult::GetRecord(Ok(libp2p::kad::GetRecordOk::FoundRecord(libp2p::kad::PeerRecord {
				peer,
				record
			}))),
			stats,
			step
		}))) = event.downcast_ref()
		&& self.query_id_to_search.contains_key(id) {
			let key: libp2p::kad::QueryId = id.to_owned();
			let entry: &mut Entry = self.query_id_to_entry.entry(key).or_insert_with(|| {
				let now: std::time::Instant = std::time::Instant::now();
				let timeout: std::time::Instant = now + self.cache_timeout;
				let timeout: Timeout = Timeout::from(timeout);
				let available: Vec<_> = Vec::default();
				let available: Available = Available::from(available);

				Entry::from((available, timeout))
			});
			if let Ok(record) = serde_json::from_slice(&record.value) {
				let record: Record = record;
				let Entry((Available(records), _)) = entry;

				records.push(record);
			}
			if step.last
			&& let Some(entry) = self.query_id_to_entry.remove(id)
			&& let Some(query) = self.query_id_to_search.remove(id) {
				self.search_to_pending_update.remove(&query);
				self.search_to_query_id.remove(&query);
				self.search_to_entry.insert(query.to_owned(), entry.to_owned());

				let Query(domain) = query;
				let Entry((Available(records), _)) = entry;
				let event: Found = Found::from((domain, available));
				let event: Event = Event::from_any(event);

				queue(event);
			}
		}
	}
}
