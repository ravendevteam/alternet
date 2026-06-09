use super::*;
use futures::SinkExt as _;
use futures::StreamExt as _;
use tokio::io::AsyncWriteExt as _;
use tokio::io::AsyncReadExt as _;
use tokio_util::compat::FuturesAsyncReadCompatExt as _;

// integrated with the event dispatch mechanism, so other sub systems can receive and send bytes, and start stream

pub const PROTOCOL: libp2p::StreamProtocol = libp2p::StreamProtocol::new("/an/0.1.0");

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct Connection {
	pub peer: libp2p::PeerId
}

#[derive(Debug)]
pub struct ConnectionEstablishment {
	pub src: libp2p::PeerId,
	pub dst: libp2p::PeerId,
	pub stream: libp2p::Stream
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
pub struct BytesInbound {
	pub src: libp2p::PeerId,
	pub content: bytes::Bytes
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct BytesOutbound {
	pub dst: libp2p::PeerId,
	pub content: bytes::Bytes
}

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
			let dst: libp2p::PeerId = swarm.local_peer_id().to_owned();
			let event_sx: tokio::sync::mpsc::Sender<_> = self.event_sx.to_owned();
			
			tokio::spawn(async move {
				while let Some((peer, stream)) = streams.next().await {
					let src: libp2p::PeerId = peer;
					let event: Event = Event::new(ConnectionEstablishment {
						src,
						dst,
						stream
					});
					
					if event_sx.send(event).await.is_err() {
					    break
				    }
				}
			});
		}
		
		if let Some(Connection {
			peer
		}) = event.downcast_ref() {
			let mut control: libp2p_stream::Control = swarm.behaviour_mut().stream.new_control().to_owned();
			let src: libp2p::PeerId = swarm.local_peer_id().to_owned();
			let dst: libp2p::PeerId = peer.to_owned();
			let event_sx: tokio::sync::mpsc::Sender<_> = self.event_sx.to_owned();
			
			tokio::spawn(async move {
				let Ok(stream) = control.open_stream(dst.to_owned(), PROTOCOL).await else {
					return
				};
				
				let event: Event = Event::new(ConnectionEstablishment {
					src,
					dst,
					stream
				});
				
				event_sx.send(event).await.ok();
			});
		}
		
		if let Some(BytesOutbound {
			dst,
			content
		}) = event.downcast_ref() 
		&& let Some(session_sx) = self.peer_to_bytes_sx.get(dst) {
			let session_sx: tokio::sync::mpsc::Sender<_> = session_sx.to_owned();
			let content: bytes::Bytes = content.to_owned();
			
			tokio::spawn(async move {
				session_sx.send(content).await.ok();
			});
		}
		
		if let Some(ConnectionEstablishment {
			src,
			dst,
			stream
		}) = event.downcast_ref() {
			let src: libp2p::PeerId = src.to_owned();
			let dst: libp2p::PeerId = dst.to_owned();
			let remote_peer: libp2p::PeerId = if src == swarm.local_peer_id().to_owned() {
				dst
			} else {
				src
			};
			let event_sx: tokio::sync::mpsc::Sender<_> = self.event_sx.to_owned();
			let stream: tokio_util::compat::Compat<_> = stream.compat();
			let stream: tokio_util::codec::Framed<_, _> = tokio_util::codec::Framed::new(stream, tokio_util::codec::LengthDelimitedCodec::default());
			let (mut stream_sx, mut stream_rx) = stream.split();
			// let (src_sx, src_rx) = tokio::sync::mpsc::channel(1000);
			let (dst_sx, dst_rx) = tokio::sync::mpsc::channel(1000);
			let mut buffer: Vec<_> = vec![0; 4000];
			
			self.peer_to_bytes_sx.insert(remote_peer, dst_sx);

			tokio::spawn(async move {
				while let Some(Ok(bytes)) = stream_rx.next().await {
					let content: bytes::Bytes = bytes.freeze();
					let event: Event = Event::new(BytesInbound {
						src: remote_peer.to_owned(),
						content
					});
					if event_sx.send(event).await.is_err() {
						break
					}
				}
				
				let event: Event = Event::new(Disconnection {
					peer: remote_peer
				});
				
				event_sx.send(event).await.ok();
			});
			
			tokio::spawn(async move {
				while let Some(bytes) = dst_rx.recv().await {
					if stream_sx.send(bytes).await.is_err() {
						break
					}
				}
			});
		}
		
		if let Some(Disconnection {
			peer
		}) = event.downcast_ref() {
			self.peer_to_bytes_sx.remove(peer);
		}
	}
}