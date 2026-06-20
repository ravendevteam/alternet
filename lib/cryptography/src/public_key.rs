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
    content: lib_bytes::NonEmpty
}

impl<T> From<lib_bytes::NonEmpty> for PublicKey<T> {
	fn from(value: lib_bytes::NonEmpty) -> Self {
		let content: lib_bytes::NonEmpty = value;
		Self {
			phantom_data: std::marker::PhantomData,
			content
		}
	}
}

impl<T> Into<lib_bytes::NonEmpty> for PublicKey<T> {
	fn into(self) -> lib_bytes::NonEmpty {
		self.content
	}
}