pub mod proto {
    include!("../proto_target/an.rs");
}

pub struct Node {

}

#[async_trait::async_trait]
impl proto::node_server::Node for Node {
    async fn ping(&self, request: tonic::Request<proto::PingRequest>) -> std::result::Result<tonic::Response<proto::PingResponse>, tonic::Status> {
        todo!()
    }
}