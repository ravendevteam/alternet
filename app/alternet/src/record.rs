use super::*;

pub struct Record {

}

impl Record {
    pub fn into_kad_record(self) -> kad::Record {
        kad::Record {
            
        }
    }
}

impl From<kad::Record> for Record {
    fn from(value: kad::Record) -> Self {
        
    }
}