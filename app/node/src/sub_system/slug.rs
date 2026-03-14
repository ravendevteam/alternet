use super::*;

pub struct Slug {
    delay: std::time::Duration
}

#[bon::bon]
impl Slug {
    #[builder]
    pub fn new(delay: std::time::Duration) -> Self {
        Self {
            delay
        }
    }
}

impl SubSystem for Slug {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
        let Some(SwarmEvent::Behaviour(BehaviourEvent::Kad(_))) = event.downcast_ref() else {
            return
        };
        if rand::random::<f32>() < 0.05 {
            log::warn!("blocking main loop for {:?}", self.delay);
            std::thread::sleep(self.delay);
        }
    }
}