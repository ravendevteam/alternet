use super::*;

pub struct An;

impl stream::Protocol for An {
	fn protocol() -> libp2p::StreamProtocol {
		libp2p::StreamProtocol::new("/an")
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct Reservation {
	owner: PublicKey,
	owner_signature: Signature,
	src: libp2p::PeerId,
	src_public_key: PublicKey,
	dst: libp2p::PeerId,
	dst_public_key: PublicKey,
	ttl: std::time::Duration
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub enum Opcode {
	Reserve(Reservation),
	Idle
}

impl std::str::FromStr for Opcode {
	type Err = ();
	
	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct Search(Domain);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct Found(Vec<libp2p::Multiaddr>);

pub struct PendingReservation {
	src: libp2p::PeerId,
	src_stream_established: bool,
	dst: libp2p::PeerId,
	dst_stream_established: bool
}

pub struct Broker<T> {
	dns: T,
	res: Vec<Reservation>,
	
	// cache visited ones
	domain_to_addrs: std::collections::HashMap<Domain, Found>
}

impl<T> Broker<T> {
	
}

impl<T> SubSystem for Broker<T> 
where
	T: Dns {
	fn receive(
		&mut self, 
		swarm: &mut Swarm, 
		event: &mut Event, 
		queue: &mut dyn FnMut(Event)
	) {
		
		
		
		let swarm = swarm.behaviour_mut();
		
		queue(Event::from_any(search_engine::Query::from(Domain::from(String::from("hello.an")))));

		if let Some(Search(domain)) = event.downcast_ref() {
			
			// after search cache result, and send out event for the entire system
			queue(Event::from_any(Found::from(vec![])));
		}
		
		// begin search for the specific key on the kad, the key should be the domain it needs to look up
		// after recieving all matches, it will verify which one actually has the real signature of the owner of the domain proving it is the real one
		// the record should contain the address to connect to
		// 
		// relay can then establish a connection with that peer, and dns search complete
		swarm.kad.get_record(libp2p::kad::RecordKey::new(b""));
		
		if let Some(SwarmEvent::Behaviour(BehaviourEvent::Kad(libp2p::kad::Event::OutboundQueryProgressed {
			id,
			result: libp2p::kad::QueryResult::GetRecord(Ok(libp2p::kad::GetRecordOk::FoundRecord(libp2p::kad::PeerRecord {
				peer,
				record
			}))),
			stats,
			step
		}))) = event.downcast_ref() {
			// look up signatures for validity
			record.value;
		}
		
		#[cfg(feature = "relay")] {
			if let Some(stream::Inbound::<An>(stream::Packet {
				peer,
				content,
				..
			})) = event.downcast_ref() {
				let content: Vec<_> = content.to_vec();
				let content: &str = std::str::from_utf8(&content).unwrap();
				let content: Opcode = content.parse().unwrap();
				if let Opcode::Reserve(Reservation {
					owner,
					owner_signature,
					src,
					src_public_key,
					dst,
					dst_public_key,
					ttl
				}) = content {
					
					
					tokio::runtime::Handle::current().block_on(async move {
						match self.dns.locked_balance_of(owner).await {
							Ok(balance) => {
								if balance < 200 {
									
								}
							},
							Err(_) => {
								
							}
						}
						
						// how long will it be valid for
						self.dns.locked_balance_timeout_of(owner).await.unwrap();
						
						// all good, create forward event and timeout for it
						queue();
					});
				}
			}	
		}
	}
}