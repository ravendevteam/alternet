use super::*;

#[derive(Debug)]
pub struct Rotation;

pub struct IdentitySpoofer {
    interval: std::time::Duration,
    last_rotation: std::time::Instant
}

#[bon::bon]
impl IdentitySpoofer {
    #[builder]
    pub fn new(interval: std::time::Duration) -> Self {
        let last_rotation: std::time::Instant = std::time::Instant::now();
        Self {
            interval,
            last_rotation
        }
    }
}

impl SubSystem for IdentitySpoofer {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
        // ... todo ...

        let Some(_) = event.downcast_ref::<SwarmEvent>() else {
            return
        };
        if self.last_rotation.elapsed() < self.interval {
            return
        }
        self.last_rotation = std::time::Instant::now();
        log::warn!("rotating peer identity");
        queue(Event::new(Rotation))
    }
}