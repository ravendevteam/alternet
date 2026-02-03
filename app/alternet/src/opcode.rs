use super::*;

pub enum Opcode {
    Resolve(Resolve),
    Put(Put)
}

pub struct Resolve {
    pub domain: String
}

pub struct Put {
    pub domain: String
}