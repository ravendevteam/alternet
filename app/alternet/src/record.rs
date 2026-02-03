use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(getset::Getters)]
pub struct Record {
    #[get = "pub"]
    domain: String,
    #[get = "pub"]
    publisher: PeerId,
    #[get = "pub"]
    expiration: std::time::Instant
}

impl Record {
    pub fn new(domain: String, publisher: PeerId, expiration: std::time::Instant) -> Self {
        Self {
            domain,
            publisher,
            expiration
        }
    }
}

impl Record {
    pub fn to_bytes(&self) -> Vec<u8> {
        let ret: &str = &self.domain;
        let ret: &[u8] = ret.as_bytes();
        let ret: Vec<u8> = ret.to_owned();
        ret
    }
}

#[allow(clippy::from_over_into)]
impl Into<kad::Record> for Record {
    fn into(self) -> kad::Record {
        let key: &str = self.domain();
        let key: Vec<u8> = key.as_bytes().to_vec();
        let key: kad::RecordKey = key.into();
        let value: Vec<u8> = self.to_bytes();
        let publisher: PeerId = self.to_owned().publisher().to_owned();
        let publisher: Option<PeerId> = Some(publisher);
        let expires: std::time::Instant = self.expiration().to_owned();
        let expires: Option<std::time::Instant> = Some(expires);
        kad::Record {
            key,
            value,
            publisher,
            expires
        }
    }
}