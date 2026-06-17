use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
#[serde(transparent)]
pub struct PublicKey<T> {
    #[serde(skip)]
    phantom_data: std::marker::PhantomData<T>,
    bytes: Bytes
}

impl<T> From<Bytes> for PublicKey<T> {
    fn from(value: Bytes) -> Self {
        let phantom_data: std::marker::PhantomData<_> = std::marker::PhantomData;
        let bytes: Bytes = value;
        Self {
            phantom_data,
            bytes
        }
    }
}

impl<T> AsRef<[u8]> for PublicKey<T> {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}