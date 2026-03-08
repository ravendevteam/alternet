use super::*;

pub struct SelfDestruct;

impl SubSystem for SelfDestruct {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
        if rand::random::<f32>() < 0.001 {
            log::warn!("self destructing");
            panic!("purposely terminated");
        }
    }
}