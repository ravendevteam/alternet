use super::*;
use futures::SinkExt as _;
use futures::StreamExt as _;
use tokio::io::AsyncWriteExt as _;
use tokio::io::AsyncReadExt as _;
use tokio_util::compat::FuturesAsyncReadCompatExt as _;

// integrated with the event dispatch mechanism, so other sub systems can receive and send bytes, and start stream

pub const PROTOCOL: libp2p::StreamProtocol = libp2p::StreamProtocol::new("/an/0.1.0");

pub struct ConnectionRequest {
	dst: libp2p::PeerId
}

pub struct ForwardRequest {
	src: libp2p::PeerId,
	dst: libp2p::PeerId
}

pub struct BytesInbound {
	src: libp2p::PeerId,
	content: bytes::BytesMut
}

pub struct BytesOutbound {
	dst: libp2p::PeerId,
	content: bytes::BytesMut
}

pub struct Send<T> {
	
}

pub enum Signal {
	Session {
		session_key: String,
		peer: libp2p::PeerId
	},
	Ready {
		src: libp2p::PeerId,
		dst: libp2p::PeerId,
		stream: libp2p::Stream
	}
}

pub struct SessionManager {
	sx: tokio::sync::mpsc::Sender<Signal>,
	rx: tokio::sync::mpsc::Receiver<Signal>,
	session_key_to_peer_id: std::collections::HashMap<u64, libp2p::PeerId>
}

impl SubSystem for SessionManager {
	fn receive(
		&mut self, 
		swarm: &mut Swarm, 
		event: &mut Event, 
		queue: &mut dyn FnMut(Event)
	) {
		if let Some(ConnectionRequest) = event.downcast_ref() {
			let mut control = swarm.behaviour_mut().stream.new_control().to_owned();
			
			tokio::spawn(async move {
				match control.open_stream("", PROTOCOL).await {
					
				}
			});
		}
		
    	while let Ok(signal) = self.rx.try_recv() {
     		match signal {
       			Signal::Session {
       				session_key,
          			peer
        		} => {
          			tokio::spawn(async move {
             			
             		});
          		},
           		Signal::Ready {
             		src,
               		dst,
             		stream
            	} => {
             		let stream: tokio_util::compat::Compat<_> = stream.compat();
             		let stream: tokio_util::codec::Framed<_> = tokio_util::codec::Framed::new(stream, tokio_util::codec::LengthDelimitedCodec::default());
             		let (mut stream_sx, mut stream_rx) = stream.split();
               		let (src_sx, src_rx) = tokio::sync::mpsc::channel(1000);
                 	let (dst_sx, dst_rx) = tokio::sync::mpsc::channel(1000);
                  	let mut buffer: Vec<_> = vec![0; 4000];
                   
                   	// src_rx => receive bytes in
                    // dst_sx => send bytes out
                
             		tokio::spawn(async move {
               			while let Some(s) = stream_rx.next().await {
                  			match s {
                     			Ok(bytes) => {
                        			if src_sx.send(bytes).await.is_err() {
                           				break
                           			}
                        		},
                          		Err(_) => {
                            		break
                            	}
                     		}
                  		}
               		});
               
               		tokio::spawn(async move {
                 		while let Some(bytes) = dst_rx.recv().await {
                   			if stream_sx.send(bytes).await.is_err() {
                      			break
                      		}
                   		}
                 	});
             	}
       		}
     	}
	}
}