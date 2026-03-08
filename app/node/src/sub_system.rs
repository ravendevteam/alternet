use super::*;

pub mod bootstrap;
pub mod connection_manager;
pub mod dht_poison;
pub mod dialer;
pub mod identity_spoofer;
pub mod discovery_monitor;
pub mod nat_observer;
pub mod peer_registry;
pub mod relay_killer;
pub mod routing_monitor;
pub mod self_destruct;
pub mod slug;

pub trait SubSystem {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event));
}

#[derive(Default)]
pub struct Bus {
    systems: Vec<Box<dyn SubSystem>>
}

impl Bus {
    pub fn add_system<T>(&mut self, system: T) 
    where
        T: SubSystem,
        T: 'static {
        self.systems.push(Box::new(system));
    }

    pub fn receive(&mut self, swarm: &mut Swarm, event: Event) {
        let mut queue: std::collections::VecDeque<_> = vec![event].into();
        while let Some(mut event) = queue.pop_front() {
            for system in self.systems.iter_mut() {
                let mut queue = |event| queue.push_back(event);
                system.receive(swarm, &mut event, &mut queue);
            }
        }
    }
}