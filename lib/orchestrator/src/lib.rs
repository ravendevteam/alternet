use p2p::*;
use async_trait::async_trait;
use anyhow::Result;
use future::StreamExt as _;

pub mod suspend;

#[async_trait]
pub trait Node
where
    Self: Sized {
    type Opcode;

    fn swarm(&self) -> &Swarm;
    fn swarm_mut(&mut self) -> &mut Swarm;
    async fn receive(&mut self, event: SwarmEvent) -> Result<(), Box<dyn std::error::Error>>;
    async fn receive_opcode(&mut self, opcode: Self::Opcode) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Runtime<T> 
where
    T: Node {
    sx: tokio::sync::mpsc::UnboundedSender<T::Opcode>
}

impl<T> Runtime<T> 
where
    T: Node {
    pub fn spawn(mut node: T) -> Self
    where
        T: Node,
        T: Send,
        T: 'static,
        T::Opcode: Send,
        T::Opcode: 'static {
        let (sx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T::Opcode>();
        tokio::spawn(async move {
            loop {
                tokio::select!(
                    opcode = rx.recv() => {
                        match opcode {
                            Some(opcode) => node.receive_opcode(opcode).await.unwrap(),
                            None => break
                        }
                    }
                    event = node.swarm_mut().select_next_some() => {
                        if let Err(error) = node.receive(event).await {
                            println!("{}", error);
                        }
                    }
                )
            }
        });
        Self {
            sx
        }
    }
}