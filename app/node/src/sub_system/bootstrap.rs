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

#[derive(Default)]
enum Mode {
    #[default]
    WaitingForPeers,
    Bootstrapping {
        query_id: kad::QueryId
    },
    TimedOut {
        next_attempt: std::time::Instant
    },
    Healthy
}

pub struct Bootstrap {
    mode: Mode,
    addrs: Vec<libp2p::Multiaddr>,
    last_attempt: Option<std::time::Instant>,
    cooldown: std::time::Duration,
    timeout_duration: std::time::Duration,
    min_peers: usize,
    dialed: bool
}

#[bon::bon]
impl Bootstrap {
    #[builder]
    pub fn new(
        #[builder(into)]
        addrs: Vec<libp2p::Multiaddr>,
        #[builder(into)]
        cooldown: std::time::Duration,
        #[builder(into)]
        timeout_duration: std::time::Duration,
        min_peers: usize
    ) -> Self {
        let mode: Mode = Mode::WaitingForPeers;
        let last_attempt: Option<_> = None;
        let dialed: bool = false;
        Self {
            mode,
            addrs,
            last_attempt,
            cooldown,
            timeout_duration,
            min_peers,
            dialed
        }
    }
}

impl Bootstrap {
    fn propagate_identify_addrs(&mut self, swarm: &mut Swarm, event: &mut Event) {
        let Some(SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received{
            connection_id,
            peer_id,
            info: identify::Info {
                public_key,
                protocol_version,
                agent_version,
                listen_addrs,
                protocols,
                observed_addr,
                signed_peer_record
            } 
        }))) = event.downcast_ref() else {
            return
        };
        let swarm: &mut Behaviour = swarm.behaviour_mut();
        for addr in listen_addrs {
            let addr: libp2p::Multiaddr = addr.to_owned();
            swarm.kad.add_address(peer_id, addr);
        }
    }
}

impl SubSystem for Bootstrap {
    fn receive(
        &mut self, 
        swarm: &mut Swarm, 
        event: &mut Event, 
        queue: &mut dyn FnMut(Event)
    ) {
        self.propagate_identify_addrs(swarm, event);
        
        #[cfg(any(
            feature = "client", 
            feature = "server", 
            feature = "relay",
            feature = "malicious_client",
            feature = "malicious_server",
            feature = "malicious_relay"
        ))]
        match &mut self.mode {
            Mode::WaitingForPeers => {
                if !self.dialed {
                    self.dialed = true;
                    for addr in &self.addrs {
                        if let Err(error) = swarm.dial(addr.to_owned()) {
                            log::warn!("failed to dial bootstrap addr {}: {:?}", addr, error);
                        } else {
                            log::info!("dialing bootstrap addr {}", addr);
                        }
                    }
                }
                if swarm.peer_count() >= self.min_peers {
                    self.mode = Mode::Healthy;
                }
                if swarm.peer_count() > 0 {
                    let now: std::time::Instant = std::time::Instant::now();
                    if let Some(last_attempt) = self.last_attempt && now.duration_since(last_attempt) < self.cooldown {
                        let remaining: std::time::Duration = self.cooldown - now.duration_since(last_attempt);
                        log::info!("bootstrap cooldown active, retry possible in {:?}", remaining);
                        return
                    }
                    match swarm.behaviour_mut().kad.bootstrap() {
                        Ok(query_id) => {
                            log::info!("starting bootstrap {}", query_id);
                            self.mode = Mode::Bootstrapping {
                                query_id
                            };
                        },
                        Err(error) => {
                            log::warn!("failed to start bootstrap: {:?}", error);
                        }
                    }
                    let local_peer_id: libp2p::PeerId = swarm.local_peer_id().to_owned();
                    swarm.behaviour_mut().kad.get_closest_peers(local_peer_id);
                }  
            },
            Mode::Bootstrapping {
                query_id
            } => {
                let Some(SwarmEvent::Behaviour(BehaviourEvent::Kad(kad::Event::OutboundQueryProgressed{
                    id,
                    result,
                    stats,
                    step
                }))) = event.downcast_ref() else {
                    return
                };
                if id != query_id {
                    return
                }
                let kad::QueryResult::Bootstrap(result) = result else {
                    return
                };
                match result {
                    Ok(kad::BootstrapOk{
                        peer,
                        num_remaining
                    }) => {
                        log::info!("bootstrap complete, remaining: {}", num_remaining);
                        self.mode = Mode::Healthy;
                    },
                    Err(error) => {
                        log::warn!("bootstrap failed: {:?}", error);
                        let next_attempt: std::time::Instant = std::time::Instant::now() + self.timeout_duration;
                        log::info!("retrying bootstrap in {:?}", self.timeout_duration);
                        self.mode = Mode::TimedOut {
                            next_attempt
                        };
                    }
                }
            },
            Mode::TimedOut {
                next_attempt
            } => {
                if std::time::Instant::now() < *next_attempt {
                    return
                }
                log::info!("bootstrap timeout expired after {:?}, retrying", self.timeout_duration);
                self.dialed = false;
                self.mode = Mode::WaitingForPeers;
                if swarm.peer_count() > 0 {
                    let local_peer_id: libp2p::PeerId = swarm.local_peer_id().to_owned();
                    swarm.behaviour_mut().kad.get_closest_peers(local_peer_id);
                }
            },
            Mode::Healthy => {
                if swarm.peer_count() >= self.min_peers {
                    return
                }
                log::info!("peer count dropped below threshold ({} < {}), retrying bootstrap in {:?}", swarm.peer_count(), self.min_peers, self.timeout_duration);
                let next_attempt: std::time::Instant = std::time::Instant::now() + self.timeout_duration;
                self.mode = Mode::TimedOut {
                    next_attempt
                };
            }
        }
    }
}