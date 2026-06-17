use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Signature<T> {
    phantom_data: std::marker::PhantomData<T>,
    #[deref]
    #[deref_mut]
    content: bytes::Bytes
}

impl<T> From<bytes::Bytes> for Signature<T> {
	fn from(value: bytes::Bytes) -> Self {
		Self {
			phantom_data: std::marker::PhantomData,
			content: value
		}
	}
}