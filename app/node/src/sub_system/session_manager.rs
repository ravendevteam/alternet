use super::*;
use futures::SinkExt as _;
use tokio_util::compat::FuturesAsyncReadCompatExt as _;

trait Bridge {
	fn bind_to_event_loop(self, peer: libp2p::PeerId, event_sx: tokio::sync::mpsc::Sender<Event>);
}

impl Bridge for libp2p::Stream {
	fn bind_to_event_loop(self, peer: libp2p::PeerId, event_sx: tokio::sync::mpsc::Sender<Event>) {
    	let stream: tokio_util::compat::Compat<_> = self.compat();
     	let stream: tokio_util::codec::Framed<_, _> = tokio_util::codec::Framed::new(stream, tokio_util::codec::LengthDelimitedCodec::default());
      	let (mut stream_sx, mut stream_rx) = stream.split();
       	let (dst_sx, mut dst_rx) = tokio::sync::mpsc::channel(1000);
		let event: Event = Event::new(RegisterPeerChannel {
			peer: peer.to_owned(),
			dst_sx
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
				let event = Event::new(InboundBytes {
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
	}
}


pub const PROTOCOL: libp2p::StreamProtocol = libp2p::StreamProtocol::new("/an/0.1.0");


#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct Connection {
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
pub struct RegisterPeerChannel {
	pub peer: libp2p::PeerId,
	pub dst_sx: tokio::sync::mpsc::Sender<bytes::Bytes>,
}


#[derive(Debug)]
pub struct SessionManager {
	peer_to_bytes_sx: std::collections::HashMap<libp2p::PeerId, tokio::sync::mpsc::Sender<bytes::Bytes>>,
	event_sx: tokio::sync::mpsc::Sender<Event>,
	event_rx: tokio::sync::mpsc::Receiver<Event>,
	boot: bool
}

impl Default for SessionManager {
	fn default() -> Self {
		let (event_sx, event_rx) = tokio::sync::mpsc::channel(1000);
		Self {
			peer_to_bytes_sx: std::collections::HashMap::default(),
			event_sx,
			event_rx,
			boot: false
		}
	}
}

impl SubSystem for SessionManager {
	fn receive(
		&mut self, 
		swarm: &mut Swarm, 
		event: &mut Event, 
		queue: &mut dyn FnMut(Event)
	) {	
		while let Ok(event) = self.event_rx.try_recv() {
			if let Some(Disconnection {
				peer
			}) = event.downcast_ref() {
				self.peer_to_bytes_sx.remove(peer);
			}
			queue(event);
		}
		
		if !self.boot {
			self.boot = true;
			let mut control: libp2p_stream::Control = swarm.behaviour_mut().stream.new_control().to_owned();
			let Ok(mut streams) = control.accept(PROTOCOL) else {
				panic!("session manager subsystem failed to accept protocol")
			};

			let event_sx: tokio::sync::mpsc::Sender<_> = self.event_sx.to_owned();
			
			tokio::spawn(async move {
				while let Some((peer, stream)) = streams.next().await {
					stream.bind_to_event_loop(peer, event_sx.to_owned());
				}
			});
		}
		
		if let Some(Connection {
			peer
		}) = event.downcast_ref() {
			let mut control: libp2p_stream::Control = swarm.behaviour_mut().stream.new_control().to_owned();
			let dst: libp2p::PeerId = peer.to_owned();
			let event_sx: tokio::sync::mpsc::Sender<_> = self.event_sx.clone();
			
			tokio::spawn(async move {
				if let Ok(stream) = control.open_stream(dst.clone(), PROTOCOL).await {
					stream.bind_to_event_loop(dst, event_sx);
				}
			});
		}
		
		if let Some(RegisterPeerChannel {
			peer,
			dst_sx
		}) = event.downcast_ref() {
		    self.peer_to_bytes_sx.insert(peer.clone(), dst_sx.clone());
		}
		
		if let Some(OutboundBytes {
			dst,
			content
		}) = event.downcast_ref() {
			if let Some(session_sx) = self.peer_to_bytes_sx.get(dst) {
				let session_sx: tokio::sync::mpsc::Sender<_> = session_sx.clone();
				let content: bytes::Bytes = content.clone();
				tokio::spawn(async move {
					let _ = session_sx.send(content).await;
				});
			}
		}
		
		if let Some(Disconnection {
			peer
		}) = event.downcast_ref() {
			self.peer_to_bytes_sx.remove(peer);
		}
	}
}