use super::*;

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Key {
    bytes: Vec<u8>
}

impl From<Vec<u8>> for Key {
    fn from(value: Vec<u8>) -> Self {
        let bytes: Vec<_> = value;
        Self {
            bytes
        }
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        let bytes: Vec<_> = value.as_bytes().to_vec();
        Self {
            bytes
        }
    }
}

impl Into<kad::RecordKey> for Key {
    fn into(self) -> kad::RecordKey {
        self.bytes.into()
    }
}