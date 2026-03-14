use super::*;

pub mod proto;

pub struct Dial {
    pub addr: libp2p::Multiaddr,
    pub completed: std::sync::Arc<tokio::sync::Mutex<Option<bool>>>
}

pub struct PeerId {
    pub completed: std::sync::Arc<tokio::sync::Mutex<Option<libp2p::PeerId>>>
}

pub struct Server {
    sx: tokio::sync::mpsc::Sender<Event>
}

impl Server {
    pub fn new(sx: tokio::sync::mpsc::Sender<Event>) -> Self {
        Self {
            sx
        }
    }
}

#[async_trait::async_trait]
impl proto::node_server::Node for Server {
    async fn peer_id(&self, request: tonic::Request<proto::PeerIdRequest>) -> std::result::Result<tonic::Response<proto::PeerIdResponse>, tonic::Status> {
        let completed = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let event = PeerId {
            completed: completed.to_owned()
        };
        let event = Event::new(event);
        self.sx.send(event).await
            .ok()
            .ok_or(tonic::Status::internal("unable to send event"))?;
        loop {
            let lock: tokio::sync::MutexGuard<_> = completed.lock().await;
            if lock.is_some() {
                break
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let response = completed.lock().await.take().unwrap();        
        let response = proto::PeerIdResponse {
            peer_id: response.to_string()
        };
        Ok(tonic::Response::new(response))
    }

    async fn ping(&self, request: tonic::Request<proto::PingRequest>) -> std::result::Result<tonic::Response<proto::PingResponse>, tonic::Status> {
        
        log::info!("command received {:?}", request);

        Ok(tonic::Response::new(proto::PingResponse{ success: false }))
    }

    async fn dial(&self, request: tonic::Request<proto::DialRequest>) -> std::result::Result<tonic::Response<proto::DialResponse>, tonic::Status> {
        log::info!("received dial request: {:?}", request);
        let request: proto::DialRequest = request.into_inner();
        let addr: libp2p::Multiaddr = request.addr
            .parse()
            .ok()
            .ok_or(tonic::Status::invalid_argument("failed to parse multiaddr"))?;
        let completed: std::sync::Arc<_> = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let event: Dial = Dial {
            addr,
            completed: completed.to_owned()
        };
        let event: Event = Event::new(event);
        self.sx.send(event).await
            .ok()
            .ok_or(tonic::Status::internal(format!("failed to send event")))?;
        loop {
            let lock: tokio::sync::MutexGuard<_> = completed.lock().await;
            if lock.is_some() {
                break
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let response: bool = completed.lock().await.take().unwrap();
        let response: proto::DialResponse = proto::DialResponse {
            success: response,
            error: None,
            connection: None
        };
        Ok(tonic::Response::new(response))
    }
}