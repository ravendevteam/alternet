use super::*;

pub trait Common {
    fn to_empty(self) -> Option<Empty>;
    fn to_domain_mapping(self: Box<Self>) -> Option<DomainMapping>;
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Domain {
    name: String,
    ip: (u8, u8, u8, u8)
}

pub type Empty = Record<IsEmpty>;
pub type DomainMapping = Record<IsDomainMapping>;

pub struct IsEmpty;
pub struct IsDomainMapping(Domain);

#[repr(transparent)]
#[derive(Debug)]
#[derive(Clone)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Record<T> {
    state: T
}

impl DomainMapping {
    pub fn n() {

    }
}

impl<T> Default for Record<T> 
where
    T: Default {
    fn default() -> Self {
        Self {
            state: T::default()
        }
    }
}

impl Common for Empty {
    fn to_empty(self: Box<Self>) -> Option<Empty> {
        Some(*self)
    }

    fn to_domain_mapping(self: Box<Self>) -> Option<Record<IsDomainMapping>> {
        None
    }
}

impl Common for DomainMapping {
    fn to_empty(self: Box<Self>) -> Option<Empty> {
        None
    }

    fn to_domain_mapping(self: Box<Self>) -> Option<Record<IsDomainMapping>> {
        Some(*self)
    }
}