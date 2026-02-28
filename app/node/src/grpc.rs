use super::*;

pub mod proto;

#[derive(Debug)]
#[derive(derive_more::From)]
pub struct Envelope {
    item: Box<dyn std::any::Any + Send>
}

impl Envelope {
    pub fn new<T>(item: T) -> Self
    where
        T: std::any::Any,
        T: Send,
        T: 'static {
        let item: Box<_> = Box::new(item);
        Self {
            item
        }
    }

    pub fn downcast_ref<T>(&self) -> Option<&T> 
    where
        T: std::any::Any {
        self.item.downcast_ref()
    }

    pub fn downcast<T>(self) -> std::result::Result<T, Self> 
    where
        T: std::any::Any {
        match self.item.downcast::<T>() {
            Ok(item) => {
                let item: T = *item;
                Ok(item)
            },
            Err(item) => {
                let item: Self = Self {
                    item
                };
                Err(item)
            }
        }
    }
}

pub struct Server {
    sx: tokio::sync::mpsc::Sender<Envelope>
}

impl Server {
    pub fn new(sx: tokio::sync::mpsc::Sender<Envelope>) -> Self {
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