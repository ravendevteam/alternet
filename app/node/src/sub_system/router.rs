use super::*;
use futures::SinkExt as _;
use tokio_util::compat::FuturesAsyncReadCompatExt as _;

// TODO: make each event generic over the protocl

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct StreamConnectionRequest {
	pub peer: libp2p::PeerId
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct SteamConnectionFailure {
	pub peer: libp2p::PeerId
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct StreamConnectionSuccess {
	pub peer: libp2p::PeerId
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct Disconnection {
	pub peer: libp2p::PeerId
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct InboundBytes<T> {
	phantom_data: std::marker::PhantomData<T>,
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
#[derive(Clone)]
pub struct PeerChannelRegistration {
	pub peer: libp2p::PeerId,
	pub dst_sx: tokio::sync::mpsc::Sender<bytes::Bytes>,
}






pub trait Protocol {
	fn protocol() -> libp2p::StreamProtocol; 
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(derive_more::From)]
pub struct Packet(libp2p::PeerId, bytes::Bytes);

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
        
        let event: Event = Event::new(PeerChannelRegistration {
        	peer,
        	dst_sx: dst_sx.to_owned()
        });
        
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
				let event = Event::new(InboundBytes::<T> {
					phantom_data: std::marker::PhantomData,
					src: peer.to_owned(),
					content: bytes.freeze(),
				});
				if event_sx.send(event).await.is_err() {
					break;
				}
			}

			event_sx.send(Event::new(Disconnection {
				peer: peer.to_owned()
			})).await;
		});
		
		dst_sx
	}
}

#[derive(Debug)]
pub struct Router<T> {
	phantom_data: std::marker::PhantomData<T>,	
	src_to_dsts: std::collections::HashMap<libp2p::PeerId, Vec<libp2p::PeerId>>,
	peer_to_blacklist: std::collections::HashMap<libp2p::PeerId, bool>,
	peer_to_bytes_sx: std::collections::HashMap<libp2p::PeerId, tokio::sync::mpsc::Sender<bytes::Bytes>>,
	event_sx: tokio::sync::mpsc::Sender<Event>,
	event_rx: tokio::sync::mpsc::Receiver<Event>,
	setup_complete: bool
}

impl<T> SubSystem for Router<T> 
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
			if let Some(PeerChannelRegistration {
				peer,
				dst_sx
			}) = event.downcast_ref() {
				self.peer_to_bytes_sx.insert(Clone::clone(&peer), Clone::clone(&dst_sx));
				
				continue
			} else if let Some(Disconnection {
				peer
			}) = event.downcast_ref() {
				self.peer_to_bytes_sx.remove(peer);
			}
			
			queue(event);
		}
		
		if let Some(StreamConnectionRequest {
			peer
		}) = event.downcast_ref() {
			let mut control: libp2p_stream::Control = swarm.behaviour_mut().stream.new_control().to_owned();
			let dst: libp2p::PeerId = peer.to_owned();
			let event_sx: tokio::sync::mpsc::Sender<_> = self.event_sx.to_owned();
			
			tokio::spawn(async move {
				match control.open_stream(ToOwned::to_owned(&dst), T::protocol()).await {
					Ok(stream) => {
						stream.bind::<T>(dst.to_owned(), event_sx.to_owned());
						
						let event: Event = Event::new(StreamConnectionSuccess {
							peer: dst.to_owned()
						});
						
						event_sx.send(event);
					},
					Err(error) => {
						let event: Event = Event::new(SteamConnectionFailure {
							peer: dst.to_owned()
						});
						
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
		
		if let Some(OutboundBytes {
			dst,
			content
		}) = event.downcast_ref() {
			if let Some(bytes_sx) = self.peer_to_bytes_sx.get(dst) {
				let bytes_sx: tokio::sync::mpsc::Sender<_> = bytes_sx.to_owned();
				let content: bytes::Bytes = content.to_owned();
				
				tokio::spawn(async move {
					bytes_sx.send(content).await.ok();
				});
			}
		}
		
		if let Some(PeerChannelRegistration {
			peer,
			dst_sx
		}) = event.downcast_ref() {
		    self.peer_to_bytes_sx.insert(ToOwned::to_owned(peer), ToOwned::to_owned(dst_sx));
		}
	}
}
