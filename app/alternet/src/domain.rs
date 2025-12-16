use super::*;

impl Behaviour {
    pub fn register_domain(&mut self) {
        // pseudo code example
        let record = kad::Record {
            key: Vec::new().into(),
            value: Vec::new(),

            // somehow pass peer_id in here, maybe through state behaviour
            publisher: Some(peer_id),
            expires: None
        };
        self.kad.put_record(record, kad::Quorum::Majority);
    }
}