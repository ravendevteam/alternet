use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct Cidr {
    addr: std::net::Ipv4Addr,
    mask: u8
}

impl Cidr {
    pub fn new(addr: std::net::Ipv4Addr, mask: u8) -> Result<Self> {
        if mask > 32 {
            return Err(anyhow!("out of bounds"))
        }
        Ok(Self { addr, mask })
    }
}

impl std::fmt::Display for Cidr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.addr, self.mask)
    }
}