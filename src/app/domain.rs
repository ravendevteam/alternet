use super::*;

impl App {
    pub fn register_domain(&mut self) {
        // pseudo code example
        let record = kad::Record {
            key: Vec::new().into(),
            value: Vec::new(),
            publisher: Some(self.peer_id),
            expires: None
        };
        self.network.kad.put_record(record, kad::Quorum::Majority);
    }
}