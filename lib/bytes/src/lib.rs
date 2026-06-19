#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct NonEmpty(bytes::Bytes);

impl TryFrom<bytes::Bytes> for NonEmpty {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: bytes::Bytes) -> Result<Self, Self::Error> {
		if value.len() == 0 {
			return Err(<Box<dyn std::error::Error>>::from(String::from("empty")))
		}
		Ok(Self(value))
	}
}

impl Into<bytes::Bytes> for NonEmpty {
	fn into(self) -> bytes::Bytes {
		self.0
	}
}