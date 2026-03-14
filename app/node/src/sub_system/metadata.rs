use super::*;

pub struct Metadata;

impl SubSystem for Metadata {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
        let Some(grpc::PeerId{
            completed
        }) = event.downcast_mut() else {
            return
        };
        let peer_id = swarm.local_peer_id().to_owned();
        let completed = completed.to_owned();
        tokio::spawn(async move {
            let mut completed = completed.lock().await;
            *completed = Some(peer_id);
        });
    }
}