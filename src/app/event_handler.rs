use super::*;

pub trait EventHandler {
    fn handle(&mut self, swarm: swarm::Swarm<Network>);
}