use super::*;
use futures::SinkExt as _;
use tokio_util::compat::FuturesAsyncReadCompatExt as _;

pub trait Protocol {
	fn protocol() -> libp2p::StreamProtocol; 
}

trait Bridge {	
	fn bind<T>(self, peer: libp2p::PeerId, event_sx: tokio::sync::mpsc::Sender<Event>) -> tokio::sync::mpsc::Sender<bytes::Bytes>
	where
		T: 'static,
		T: Send;
}

impl Bridge for libp2p::Stream {
	fn bind<T>(self, peer: libp2p::PeerId, event_sx: tokio::sync::mpsc::Sender<Event>) -> tokio::sync::mpsc::Sender<bytes::Bytes> 
	where
		T: 'static,
		T: Send {
    	let stream: tokio_util::compat::Compat<_> = self.compat();
     	let stream: tokio_util::codec::Framed<_, _> = tokio_util::codec::Framed::new(stream, tokio_util::codec::LengthDelimitedCodec::default());
      	let (mut stream_sx, mut stream_rx) = stream.split();
       	let (dst_sx, mut dst_rx) = tokio::sync::mpsc::channel(1000);
        let event: Event = Event::from_any(Registration::<T>::from((peer, dst_sx.to_owned())));
        
        event_sx.try_send(event).ok();
		
		tokio::spawn(async move {
			while let Some(bytes) = dst_rx.recv().await {
				if stream_sx.send(bytes).await.is_err() {
					break;
				}
			}
		});
		
		tokio::spawn(async move {
			while let Some(Ok(bytes)) = stream_rx.next().await {
				let event: Event = Event::from_any(Inbound::<T>::from(Packet::from((peer.to_owned(), bytes.freeze()))));
				
				if event_sx.send(event).await.is_err() {
					break;
				}
			}

			event_sx.send(Event::from_any(Disconnect::<T>::from(Peer::from(peer)))).await;
		});
		
		dst_sx
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Peer<T> {
	phantom_data: std::marker::PhantomData<T>,
	#[deref]
	#[deref_mut]
	pub peer: libp2p::PeerId
}

impl<T> From<libp2p::PeerId> for Peer<T> {
	fn from(value: libp2p::PeerId) -> Self {
    	Self {
  			phantom_data: std::marker::PhantomData,
    		peer: value
     	}
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct StreamConnectionRequest<T>(pub Peer<T>);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct StreamConnectionFailure<T>(pub Peer<T>);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct StreamConnectionSuccess<T>(pub Peer<T>);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct Disconnect<T>(Peer<T>);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Packet<T> {
	phantom_data: std::marker::PhantomData<T>,
	pub peer: libp2p::PeerId,
	#[deref]
	#[deref_mut]
	pub content: bytes::Bytes
}

impl<T> From<(libp2p::PeerId, bytes::Bytes)> for Packet<T> {
	fn from(value: (libp2p::PeerId, bytes::Bytes)) -> Self {
		Self {
			phantom_data: std::marker::PhantomData,
			peer: value.0,
			content: value.1
		}
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
pub struct Inbound<T>(pub Packet<T>);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
pub struct Outbound<T>(pub Packet<T>);

#[derive(Debug)]
#[derive(Clone)]
pub struct Registration<T> {
	phantom_data: std::marker::PhantomData<T>,
	pub peer: libp2p::PeerId,
	pub dst_sx: tokio::sync::mpsc::Sender<bytes::Bytes>,
}

impl<T> From<(libp2p::PeerId, tokio::sync::mpsc::Sender<bytes::Bytes>)> for Registration<T> {
	fn from(value: (libp2p::PeerId, tokio::sync::mpsc::Sender<bytes::Bytes>)) -> Self {
		Self {
			phantom_data: std::marker::PhantomData,
			peer: value.0,
			dst_sx: value.1
		}
	}
}

#[derive(Debug)]
pub struct Stream<T> {
	phantom_data: std::marker::PhantomData<T>,	
	src_to_dsts: std::collections::HashMap<libp2p::PeerId, Vec<libp2p::PeerId>>,
	peer_to_bytes_sx: std::collections::HashMap<libp2p::PeerId, tokio::sync::mpsc::Sender<bytes::Bytes>>,
	pending: std::collections::HashSet<libp2p::PeerId>,
	event_sx: tokio::sync::mpsc::Sender<Event>,
	event_rx: tokio::sync::mpsc::Receiver<Event>,
	setup_complete: bool
}

impl<T> SubSystem for Stream<T> 
where
	T: 'static,
	T: Send,
	T: Protocol {
	fn receive(
		&mut self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) {
		if !self.setup_complete {
			self.setup_complete = true;
			
			log::info!("establishing listener for {}", T::protocol());
			
			let event_sx: tokio::sync::mpsc::Sender<_> = self.event_sx.to_owned();
			let mut control: libp2p_stream::Control = swarm.behaviour_mut().stream.new_control().to_owned();
			let mut streams: libp2p_stream::IncomingStreams = control.accept(T::protocol()).expect("established listener");
			
			tokio::spawn(async move {
				while let Some((peer, stream)) = streams.next().await {
					stream.bind::<T>(peer, event_sx.to_owned());
				}
			});
		}
		
		while let Ok(event) = self.event_rx.try_recv() {
			if let Some(Registration::<T> {
				peer,
				dst_sx,
				..
			}) = event.downcast_ref() {
				self.peer_to_bytes_sx.insert(Clone::clone(&peer), Clone::clone(&dst_sx));
				
				continue
			}
			
			if let Some(Disconnect::<T>(Peer {
				peer,
				..
			})) = event.downcast_ref() {
				self.peer_to_bytes_sx.remove(peer);
			}
			
			queue(event);
		}
		
		if let Some(StreamConnectionRequest::<T>(Peer {
			peer,
			..
		})) = event.downcast_ref() {
			let mut control: libp2p_stream::Control = swarm.behaviour_mut().stream.new_control().to_owned();
			let dst: libp2p::PeerId = peer.to_owned();
			let event_sx: tokio::sync::mpsc::Sender<_> = self.event_sx.to_owned();
			
			tokio::spawn(async move {
				match control.open_stream(ToOwned::to_owned(&dst), T::protocol()).await {
					Ok(stream) => {
						stream.bind::<T>(dst.to_owned(), event_sx.to_owned());
						
						let event: Event = Event::from_any(StreamConnectionSuccess::<T>::from(Peer::from(dst.to_owned())));
						
						if event_sx.send(event).await.is_err() {
							log::error!("failed to send connection success event");
						}
					},
					Err(error) => {
						let event: Event = Event::from_any(StreamConnectionFailure::<T>::from(Peer::from(dst.to_owned())));
						
						match event_sx.send(event).await {
							Ok(_) => return,
							Err(_) => {
								log::error!("attempted to send a connection failure");
							}
						}
					}
				}
			});
		}
		
		if let Some(Outbound::<T>(Packet {
			peer,
			content,
			..
		})) = event.downcast_ref() {
			if let Some(bytes_sx) = self.peer_to_bytes_sx.get(peer) {
				let bytes_sx: tokio::sync::mpsc::Sender<_> = bytes_sx.to_owned();
				let content: bytes::Bytes = content.to_owned();
				
				tokio::spawn(async move {
					bytes_sx.send(content).await.ok();
				});
			} else {
				// attempt to establish a stream with the outbound peer

				if self.pending.insert(peer.to_owned()) {
					let mut control: libp2p_stream::Control = swarm.behaviour_mut().stream.new_control().to_owned();
					let dst: libp2p::PeerId = peer.to_owned();
					let event_sx: tokio::sync::mpsc::Sender<_> = self.event_sx.to_owned();
					
					tokio::spawn(async move {
						match control.open_stream(ToOwned::to_owned(&dst), T::protocol()).await {
							Ok(stream) => {
								stream.bind::<T>(dst.to_owned(), event_sx.to_owned());
								
								let event: Event = Event::from_any(StreamConnectionSuccess::<T>::from(Peer::from(dst.to_owned())));
								
								if event_sx.send(event).await.is_err() {
									log::error!("failed to send connection success event");
								}
							},
							Err(error) => {
								let event: Event = Event::from_any(StreamConnectionFailure::<T>::from(Peer::from(dst.to_owned())));
								
								match event_sx.send(event).await {
									Ok(_) => return,
									Err(_) => {
										log::error!("attempted to send a connection failure");
									}
								}
							}
						}
					});
				}
			}
		}
	}
}
