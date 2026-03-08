use super::*;

pub struct DhtPoison;

impl SubSystem for DhtPoison {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
        let Some(SwarmEvent::Behaviour(BehaviourEvent::Kad(kad::Event::RoutingUpdated{
            peer,
            is_new_peer,
            addresses,
            bucket_range,
            old_peer
        }))) = event.downcast_ref() else {
            return
        };
        for _ in 0..10 {
            let fake_peer: libp2p::PeerId = libp2p::PeerId::random();
            let fake_addr: libp2p::Multiaddr = "/ip4/8.8.8.8/udp/4001/quic-v1".parse().unwrap();
            swarm
                .behaviour_mut()
                .kad
                .add_address(&fake_peer, fake_addr);
            log::warn!("injected fake peer {:?}", fake_peer);
        }
    }
}