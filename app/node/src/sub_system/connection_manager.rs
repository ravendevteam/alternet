use super::*;

pub enum Health {
    Stable,
    Recovering
}

pub struct Peer {
    pub connected: bool,
    pub dialing: bool,
    pub failed_dials: u32,
    pub last_attempt: std::time::Instant,
    pub last_connected: Option<std::time::Instant>
}

#[bon::bon]
impl Peer {
    #[builder]
    pub fn new(
        connected: bool,
        dialing: bool,
        failed_dials: u32,
        last_attempt: std::time::Instant,
        last_connected: Option<std::time::Instant>
    ) -> Self {
        Self {
            connected,
            dialing,
            failed_dials,
            last_attempt,
            last_connected
        }
    }
}

impl Default for Peer {
    fn default() -> Self {
        Self::builder()
            .connected(false)
            .dialing(false)
            .failed_dials(0)
            .last_attempt(std::time::Instant::now())
            .build()
    }
}

pub struct ConnectionManager {
    peers: std::collections::HashMap<libp2p::PeerId, Peer>,
    target_peer_count: usize,
    min_retry_delay: std::time::Duration,
    max_retry_delay: std::time::Duration
}

#[bon::bon]
impl ConnectionManager {
    #[builder]
    pub fn new(
        target_peer_count: usize,
        min_retry_delay: std::time::Duration,
        max_retry_delay: std::time::Duration
    ) -> Self {
        let peers: std::collections::HashMap<_, _, _> = std::collections::HashMap::default();
        Self {
            peers,
            target_peer_count,
            max_retry_delay,
            min_retry_delay
        }
    }
}

impl ConnectionManager {
    fn connected(&self) -> usize {
        self
            .peers
            .values()
            .filter(|peer| peer.connected)
            .count()
    }

    fn retry_delay(
        min_retry_delay: std::time::Duration,
        max_retry_delay: std::time::Duration,
        failed: u32
    ) -> std::time::Duration {
        let min: std::time::Duration = min_retry_delay;
        let max: std::time::Duration = max_retry_delay;
        let exp: u32 = failed.min(5);
        let delay: std::time::Duration = min * 2u32.pow(exp);
        delay.min(max)
    }

    fn maintain_target(&mut self, swarm: &mut Swarm) {
        let connected: usize = self.connected();
        if connected >= self.target_peer_count {
            return
        }
        let now: std::time::Instant = std::time::Instant::now();
        let mut required: usize = self.target_peer_count - connected;
        for (peer_id, peer) in self.peers.iter_mut() {
            if required == 0 {
                break
            }
            if peer.failed_dials > 32 || peer.connected || peer.dialing {
                continue
            }
            let delay: std::time::Duration = Self::retry_delay(self.min_retry_delay, self.max_retry_delay, peer.failed_dials);
            if now.duration_since(peer.last_attempt) < delay {
                continue
            }
            if swarm.dial(*peer_id).is_ok() {
                peer.dialing = true;
                peer.last_attempt = now;
                required -= 1;
            }
        }
    }
}

impl SubSystem for ConnectionManager {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
        if let Some(SwarmEvent::Behaviour(BehaviourEvent::Kad(kad::Event::RoutingUpdated{
            peer,
            is_new_peer,
            addresses,
            bucket_range,
            old_peer
        }))) = event.downcast_ref() {
            self.peers.entry(*peer).or_insert_with(Peer::default);
        }
        if let Some(SwarmEvent::ConnectionEstablished{
            peer_id,
            concurrent_dial_errors,
            endpoint,
            connection_id,
            num_established,
            established_in
        }) = event.downcast_ref() {
            self.peers.entry(*peer_id).or_insert_with(|| {
                Peer::builder()
                    .connected(true)
                    .dialing(false)
                    .failed_dials(0)
                    .last_attempt(std::time::Instant::now())
                    .last_connected(std::time::Instant::now())
                    .build()
            });
        }
        if let Some(SwarmEvent::ConnectionClosed{
            peer_id,
            connection_id,
            endpoint,
            num_established,
            cause
        }) = event.downcast_ref() 
        && let Some(peer) = self.peers.get_mut(peer_id) {
            peer.connected = false;
            peer.dialing = false;
        }
        if let Some(SwarmEvent::OutgoingConnectionError{
            connection_id,
            peer_id,
            error
        }) = event.downcast_ref() 
        && let Some(peer_id) = peer_id 
        && let Some(peer) = self.peers.get_mut(peer_id) {
            peer.dialing = false;
            peer.failed_dials += 1;
            peer.last_attempt = std::time::Instant::now();
        }
        self.maintain_target(swarm);
    }
}