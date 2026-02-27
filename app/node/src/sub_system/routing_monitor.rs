use super::*;

trait SwarmExt {
    fn peer_count(&mut self) -> usize;
}

impl SwarmExt for Swarm {
    fn peer_count(&mut self) -> usize {
        self
            .behaviour_mut()
            .kad
            .kbuckets()
            .map(|bucket| bucket.num_entries())
            .sum()
    }
}

pub struct RoutingMonitor {
    last_sample: std::time::Instant,
    last_peer_count: usize,
    churn_counter: usize,
    churn_window_start: std::time::Instant,
    churn_window: std::time::Duration,
    collapse_threshold: usize,
    sample_interval: std::time::Duration
}

#[bon::bon]
impl RoutingMonitor {
    #[builder]
    pub fn new(sample_interval: std::time::Duration, churn_window: std::time::Duration, collapse_threshold: usize) -> Self {
        let now: std::time::Instant = std::time::Instant::now();
        let last_sample: std::time::Instant = now;
        let last_peer_count: usize = 0;
        let churn_counter: usize = 0;
        let churn_window_start: std::time::Instant = now;
        Self {
            last_sample,
            last_peer_count,
            churn_counter,
            churn_window_start,
            churn_window,
            collapse_threshold,
            sample_interval
        }
    }
}

impl RoutingMonitor {
    fn sample(&mut self, swarm: &mut Swarm) {
        let now: std::time::Instant = std::time::Instant::now();   
        if now.duration_since(self.last_sample) < self.sample_interval {
            return
        }
        let peer_count: usize = swarm.peer_count();
        let delta: usize = peer_count.abs_diff(self.last_peer_count);
        if peer_count < self.collapse_threshold {
            log::warn!("routing collapse detected: {}", peer_count);
        }
        if delta > 20 {
            log::warn!("routing oscillation: {}", delta);
        }
        self.last_peer_count = peer_count;
        self.last_sample = now;
        if now.duration_since(self.churn_window_start) > self.churn_window {
            log::info!("routing churn: {}", self.churn_counter);
            self.churn_counter = 0;
            self.churn_window_start = now;
        }
    }
}

impl SubSystem for RoutingMonitor {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
        if let Some(SwarmEvent::Behaviour(BehaviourEvent::Kad(kad::Event::RoutingUpdated{
            peer,
            is_new_peer,
            addresses,
            bucket_range,
            old_peer
        }))) = event.downcast_ref() {
            self.churn_counter += 1;
        }
        if let Some(SwarmEvent::ConnectionClosed{
            peer_id,
            connection_id,
            endpoint,
            num_established,
            cause
        }) = event.downcast_ref() {
            self.churn_counter += 1;
        }
        self.sample(swarm);
    }
}