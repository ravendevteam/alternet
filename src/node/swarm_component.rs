use super::*;

pub trait SwarmComponent {
    fn apply(&mut self, swarm: swarm::Swarm<Network>);
}