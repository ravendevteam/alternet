use super::*;

pub mod client;
pub mod relay;

pub struct UnsetProtocol;
pub struct UnsetAlgorithm;
pub struct UnsetDns;

#[derive(Debug)]
#[derive(Clone)]
struct Route<T = UnsetProtocol> {
	phantom_data: std::marker::PhantomData<T>,
	src: libp2p::PeerId,
	dst: libp2p::PeerId
}

impl<T> TryFrom<lib_bytes::NonEmpty> for Route<T> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: lib_bytes::NonEmpty) -> std::result::Result<Self, Self::Error> {
		let bytes: bytes::Bytes = value.into();
		let bytes: Vec<_> = bytes.to_vec();
		
		let buffer: String = String::from_utf8(bytes)?;
		
		let mut segments: std::str::SplitWhitespace = buffer.split_whitespace();
		let src: &str = segments.next().ok_or("missing source peer id")?;
		let src: libp2p::PeerId = src.parse()?;
		let dst: &str = segments.next().ok_or("missing destination peer id")?;
		let dst: libp2p::PeerId = dst.parse()?;
		let out: Self = Self {
			phantom_data: std::marker::PhantomData,
			src,
			dst
		};
		Ok(out)
	}
}

impl<T> TryInto<lib_bytes::NonEmpty> for Route<T> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_into(self) -> std::result::Result<lib_bytes::NonEmpty, Self::Error> {
		let src: String = self.src.to_string();
		let dst: String = self.dst.to_string();
		
		let mut buffer: String = String::default();
		buffer.push_str(&src);
		buffer.push_str(" ");
		buffer.push_str(&dst);
		buffer.push_str(" ");
		
		let bytes: Vec<_> = buffer.into_bytes();
		let bytes: bytes::Bytes = bytes.into();
		let bytes: lib_bytes::NonEmpty = bytes.try_into()?;
		
		Ok(bytes)
	}
}

#[derive(Debug)]
#[derive(Clone)]
struct Status<T = UnsetProtocol> {
	phantom_data: std::marker::PhantomData<T>,
	code: u8
}

impl<T> TryFrom<lib_bytes::NonEmpty> for Status<T> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: lib_bytes::NonEmpty) -> std::result::Result<Self, Self::Error> {
		let bytes: bytes::Bytes = value.into();
		let bytes: Vec<_> = bytes.to_vec();
		
		let buffer: String = String::from_utf8(bytes)?;
		
		let mut segments: std::str::SplitWhitespace = buffer.split_whitespace();
		let code: &str = segments.next().ok_or("missing code")?;
		let code: u8 = code.parse()?;
		let out: Self = Self {
			phantom_data: std::marker::PhantomData,
			code
		};
		Ok(out)
	}
}

impl<T> TryInto<lib_bytes::NonEmpty> for Status<T> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_into(self) -> std::result::Result<lib_bytes::NonEmpty, Self::Error> {
		let code: u8 = self.code;
		let code: String = code.to_string();
		
		let mut buffer: String = String::default();
		buffer.push_str(&code);
		
		let bytes: Vec<_> = buffer.into_bytes();
		let bytes: bytes::Bytes = bytes.into();
		let bytes: lib_bytes::NonEmpty = bytes.try_into()?;
		
		Ok(bytes)
	}
}