use super::*;

pub mod bootstrap;
pub mod connection_manager;
pub mod nat_observer;
pub mod peer_registry;
pub mod routing_monitor;

pub trait SubSystem {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event));
}

#[derive(Debug)]
#[derive(derive_more::From)]
pub struct Event {
    item: Box<dyn std::any::Any + Send>
}

impl Event {
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