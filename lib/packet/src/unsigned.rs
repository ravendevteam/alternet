use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Unsigned<T = IsUnsetProtocol> {
	phantom_data: std::marker::PhantomData<T>,
	#[deref]
	#[deref_mut]
	content: lib_bytes::NonEmpty
}

impl<T> From<lib_bytes::NonEmpty> for Unsigned<T> {
	fn from(value: lib_bytes::NonEmpty) -> Self {
		Self {
			phantom_data: std::marker::PhantomData,
			content: value
		}
	}
}

impl<T> From<lib_cryptography::message::Message> for Unsigned<T> {
	fn from(value: lib_cryptography::message::Message) -> Self {
		let out: lib_bytes::NonEmpty = value.into();
		let out: Self = out.into();
		out
	}
}

impl<T> TryFrom<bytes::Bytes> for Unsigned<T> {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: bytes::Bytes) -> Result<Self, Self::Error> {
		let out: lib_bytes::NonEmpty = value.try_into()?;
		let out: Self = out.into();
		Ok(out)
	}
}

impl<T> Into<lib_bytes::NonEmpty> for Unsigned<T> {
	fn into(self) -> lib_bytes::NonEmpty {
		self.content
	}
}