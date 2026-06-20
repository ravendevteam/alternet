use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
pub struct Message(lib_bytes::NonEmpty);

impl Into<lib_bytes::NonEmpty> for Message {
	fn into(self) -> lib_bytes::NonEmpty {
		self.0
	}
}