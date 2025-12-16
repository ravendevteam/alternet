use super::*;

pub struct ConnectionEventHandler;

#[::async_trait::async_trait]
impl EventHandlerExt for ConnectionEventHandler {
    async fn handle(&mut self, swarm: &mut Swarm, event: &SwarmEvent) {
        match event {
            SwarmEvent::ConnectionEstablished {
                peer_id, 
                connection_id, 
                endpoint, 
                num_established, 
                concurrent_dial_errors, 
                established_in 
            } => {
                
            },
            SwarmEvent::ConnectionClosed {
                peer_id, 
                connection_id, 
                endpoint, 
                num_established, 
                cause
            } => {
                
            },
            _ => {}
        }
    }
}