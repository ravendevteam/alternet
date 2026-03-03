use super::*;

pub struct DiscoveryMonitor {
    last_report: std::time::Instant,
    interval: std::time::Duration
}

#[bon::bon]
impl DiscoveryMonitor {
    #[builder]
    pub fn new(interval: std::time::Duration) -> Self {
        let last_report: std::time::Instant = std::time::Instant::now() - interval;
        Self {
            last_report,
            interval
        }
    }
}

impl SubSystem for DiscoveryMonitor {
    fn receive(
        &mut self, 
        swarm: &mut Swarm, 
        event: &mut Event, 
        queue: &mut dyn FnMut(Event)
    ) {
        if self.last_report.elapsed() < self.interval {
            return
        }
        let local_peer_id = swarm.local_peer_id().to_owned();
        let mut known = vec![];
        let swarm = swarm.behaviour_mut();
        for bucket in swarm.kad.kbuckets() {
            for item in bucket.iter() {
                known.push(item.node.key.preimage().to_string());
            }
        }
        if known.is_empty() {
            log::info!("peer {} currently knows no one", local_peer_id);
        } else {
            log::info!("peer {} knows about {} peers: [{}]", local_peer_id, known.len(), known.join(", "));
        }
        self.last_report = std::time::Instant::now();
    }
}