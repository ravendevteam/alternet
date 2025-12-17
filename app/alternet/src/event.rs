use super::*;

pub enum Event {
    Ping(ping::Event),
    Kad(kad::Event),
    GossipSub(gossipsub::Event)
}

impl From<ping::Event> for Event {
    fn from(value: ping::Event) -> Self {
        Self::Ping(value)
    }
}

impl From<kad::Event> for Event {
    fn from(value: kad::Event) -> Self {
        Self::Kad(value)
    }
}

impl From<gossipsub::Event> for Event {
    fn from(value: gossipsub::Event) -> Self {
        Self::GossipSub(value)
    }
}