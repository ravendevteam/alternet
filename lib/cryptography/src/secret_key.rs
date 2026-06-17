use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct SecretKey<T> {
    phantom_data: std::marker::PhantomData<T>,
    #[deref]
    #[deref_mut]
    content: bytes::Bytes
}

impl<T> SecretKey<T> 
where
	T: AsymmetricKeyDerivationAlgorithm {
	pub fn public_key(&self) -> public_key::PublicKey<T> {
		T::public_key(&self)
	}
}

impl<T> From<bytes::Bytes> for SecretKey<T> {
	fn from(value: bytes::Bytes) -> Self {
		Self {
			phantom_data: std::marker::PhantomData,
			content: value
		}
	}
}