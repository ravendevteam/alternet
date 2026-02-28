use super::*;

pub mod proto;

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
    async fn ping(&self, request: tonic::Request<proto::PingRequest>) -> std::result::Result<tonic::Response<proto::PingResponse>, tonic::Status> {
        todo!()
    }
}