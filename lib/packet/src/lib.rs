#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct MarkedSignedVerified<A, B = ()> {
	phantom_data: std::marker::PhantomData<B>,
	signer: cryptography::public_key::PublicKey<A>,
	signature: cryptography::signature::Signature<A>,
	#[deref]
	#[deref_mut]
	content: bytes::Bytes
}

impl<A, B> kore::Unpack<(cryptography::signature::Signature<A>, cryptography::public_key::PublicKey<A>, Unsigned<B>)> for MarkedSignedVerified<A, B> {
	fn unpack(self) -> (cryptography::signature::Signature<A>, cryptography::public_key::PublicKey<A>, Unsigned<B>) {
		
	}
}

impl<A, B> TryFrom<MarkedSignedUnverified<A, B>> for MarkedSignedVerified<A, B> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: MarkedSignedUnverified<A, B>) -> Result<Self, Self::Error> {
		
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct MarkedSignedUnverified<A, B = ()> {
	phantom_data: std::marker::PhantomData<B>,
	signer: cryptography::public_key::PublicKey<A>,
	signature: cryptography::signature::Signature<A>,
	#[deref]
	#[deref_mut]
	content: bytes::Bytes
}

impl<A, B> MarkedSignedUnverified<A, B> {
	pub fn verify(self) -> kore::Result<MarkedSignedVerified<A, B>> {
		self.try_into()
	}
}

impl<A, B> kore::Unpack<(cryptography::signature::Signature<A>, cryptography::public_key::PublicKey<A>, Unsigned<B>)> for MarkedSignedUnverified<A, B> {
	fn unpack(self) -> (cryptography::signature::Signature<A>, cryptography::public_key::PublicKey<A>, Unsigned<B>) {
		
	}
}

impl<A, B> From<(Unsigned, &cryptography::secret_key::SecretKey<A>)> for MarkedSignedUnverified<A, B> 
where
	A: cryptography::AsymmetricSignatureAlgorithm {
	fn from(value: (Unsigned, &cryptography::secret_key::SecretKey<A>)) -> Self {
		
	}
}

impl<A, B> TryFrom<bytes::Bytes> for MarkedSignedUnverified<A, B> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: bytes::Bytes) -> Result<Self, Self::Error> {
		// ...
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Unsigned<T = ()> {
	phantom_data: std::marker::PhantomData<T>,
	#[deref]
	#[deref_mut]
	content: bytes::Bytes
}

impl<T> TryFrom<bytes::Bytes> for Unsigned<T> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: bytes::Bytes) -> Result<Self, Self::Error> {
		if value.len() == 0 {
			return Err(<Box<dyn std::error::Error>>::from(String::from("empty")))
		}
		Ok(Self(value))
	}
}