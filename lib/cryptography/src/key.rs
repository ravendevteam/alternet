use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Key<T> {
    phantom_data: std::marker::PhantomData<T>,
    #[deref]
    #[deref_mut]
    content: bytes::Bytes
}

impl<T> Key<T> 
where
    T: SymmetricKeyGenAlgorithm {
    pub fn generate() -> Result<Self> {
        T::generate()
    }
}

impl<T> Key<T>
where
    T: SymmetricEncryptionAlgorithm {
    pub fn encrypt(&self, message: message::Message) -> Result<encrypted::Encrypted<T>> {
        T::encrypt(self, message)
    }

    pub fn decrypt(&self, message: encrypted::Encrypted<T>) -> Result<message::Message> {
        T::decrypt(self, message)
    }
}