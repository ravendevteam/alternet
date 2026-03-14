use super::*;

pub struct RelayKiller;

impl SubSystem for RelayKiller {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
        let Some(SwarmEvent::ConnectionEstablished{
            peer_id,
            connection_id,
            endpoint,
            num_established,
            concurrent_dial_errors,
            established_in
        }) = event.downcast_ref() else {
            return
        };
        if rand::random::<f32>() < 0.01 {
            log::warn!("dropping relay peer {:?}", peer_id);
            let peer_id: libp2p::PeerId = peer_id.to_owned();
            swarm.disconnect_peer_id(peer_id).ok();
        }
    }
}