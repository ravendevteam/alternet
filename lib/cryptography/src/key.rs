use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
#[serde(transparent)]
pub struct Key<T> {
    #[serde(skip)]
    phantom_data: std::marker::PhantomData<T>,
    bytes: Bytes
}

impl<T> Key<T> 
where
    T: SymmetricKeyGenAlgorithm {
    pub fn generate() -> Result<Self> {
        let phantom_data: std::marker::PhantomData<_> = std::marker::PhantomData;
        let bytes: Bytes = T::generate()?;
        let new: Self = Self {
            phantom_data,
            bytes
        };
        Ok(new)
    }
}

impl<T> Key<T>
where
    T: SymmetricEncryptionAlgorithm {
    pub fn encrypt(&self, content: &Bytes) -> Result<Bytes> {
        T::encrypt(&self.bytes, content)
    }

    pub fn decrypt(&self, content: &encrypted::Encrypted<T>) -> Result<Bytes> {
        let content: Bytes = content
            .as_ref()
            .to_vec()
            .into();
        T::decrypt(&self.bytes, &content)
    }
}