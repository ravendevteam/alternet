use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct PublicKey<T> {
    phantom_data: std::marker::PhantomData<T>,
    #[deref]
    #[deref_mut]
    content: bytes::Bytes
}

impl<T> TryFrom<bytes::Bytes> for PublicKey<T> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: bytes::Bytes) -> std::result::Result<Self, Self::Error> {
		if value.len() == 0 {
			return Err(<Box<dyn std::error::Error>>::from(String::from("must not be empty")))
		}
		Ok(Self {
			phantom_data: std::marker::PhantomData,
			content: value
		})
	}
}