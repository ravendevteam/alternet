#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Message(bytes::Bytes);

impl TryFrom<bytes::Bytes> for Message {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: bytes::Bytes) -> std::result::Result<Self, Self::Error> {
		if value.len() == 0 {
			return Err(<Box<dyn std::error::Error>>::from(String::from("too short: must not be empty")))
		}
		Ok(Self(value))
	}
}