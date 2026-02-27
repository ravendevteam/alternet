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

#[derive(Default)]
pub struct Bootstrap {
    mode: Mode,
    bootstrap_addrs: Vec<libp2p::Multiaddr>,
    timeout_duration: std::time::Duration,
    min_peers: usize
}

#[bon::bon]
impl Bootstrap {
    #[builder]
    pub fn new(
        bootstrap_addrs: Vec<libp2p::Multiaddr>,
        timeout_duration: std::time::Duration,
        min_peers: usize
    ) -> Self {
        let mode: Mode = Mode::WaitingForPeers;
        Self {
            mode,
            bootstrap_addrs,
            timeout_duration,
            min_peers
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
        
        #[cfg(any(feature = "client", feature = "server", feature = "relay"))]
        match &mut self.mode {
            Mode::WaitingForPeers => {
                if swarm.peer_count() >= self.min_peers {
                    self.mode = Mode::Healthy;
                }
                if swarm.peer_count() > 0 || !self.bootstrap_addrs.is_empty() {
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
                        let next_attempt: std::time::Instant = std::time::Instant::now() + self.timeout_duration;
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
                log::info!("retrying bootstap");
                self.mode = Mode::WaitingForPeers;
            },
            Mode::Healthy => {
                if swarm.peer_count() >= self.min_peers {
                    return
                }
                log::info!("peer count dropped below threshold ({} < {})", swarm.peer_count(), self.min_peers);
                self.mode = Mode::WaitingForPeers;
            }
        }
    }
}