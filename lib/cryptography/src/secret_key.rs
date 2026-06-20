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
    content: lib_bytes::NonEmpty
}

impl<T> SecretKey<T> 
where
	T: AsymmetricKeyDerivationAlgorithm {
	pub fn public_key(&self) -> public_key::PublicKey<T> {
		T::public_key(&self)
	}
}

impl<T> From<lib_bytes::NonEmpty> for SecretKey<T> {
	fn from(value: lib_bytes::NonEmpty) -> Self {
		let content: lib_bytes::NonEmpty = value;
		Self {
			phantom_data: std::marker::PhantomData,
			content
		}
	}
}

impl<T> Into<lib_bytes::NonEmpty> for SecretKey<T> {
	fn into(self) -> lib_bytes::NonEmpty {
		self.content
	}
}

impl<T> Into<lib_bytes::NonEmpty> for &SecretKey<T> {
	fn into(self) -> lib_bytes::NonEmpty {
		self.content.to_owned()
	}
}