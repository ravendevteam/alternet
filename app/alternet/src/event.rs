use super::*;

pub enum Event {
    Ping(ping::Event),
    Kad(kad::Event)
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