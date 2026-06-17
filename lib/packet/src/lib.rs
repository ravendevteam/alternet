use bytes::Buf as _;

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

impl<A, B> TryFrom<(Unsigned<B>, &cryptography::secret_key::SecretKey<A>)> for MarkedSignedUnverified<A, B> 
where
	A: cryptography::AsymmetricSignatureAlgorithm {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: (Unsigned<B>, &cryptography::secret_key::SecretKey<A>)) -> Result<Self, Self::Error> {
		let (unsigned, secret_key) = value;
		let signature = A::sign(&secret_key.as_ref().to_vec().into_boxed_slice(), &unsigned.content.to_vec().into_boxed_slice())?;
		
	}
}

impl<A, B> TryFrom<bytes::Bytes> for MarkedSignedUnverified<A, B> 
where
	A: cryptography::AsymmetricSetLayout {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(mut value: bytes::Bytes) -> Result<Self, Self::Error> {
		let header_len: usize = A::PUBLIC_KEY_LEN + A::SIGNATURE_LEN;
		if value.len() < header_len {
			return Err(<Box<dyn std::error::Error>>::from(String::from("packet too short: insufficient header len")))
		}
		let signer: cryptography::public_key::PublicKey<A> = value.split_to(A::PUBLIC_KEY_LEN).to_vec().into_boxed_slice().into();
		let signature: cryptography::signature::Signature<A> = value.split_to(A::SIGNATURE_LEN).to_vec().into_boxed_slice().into();
		Ok(Self {
			phantom_data: std::marker::PhantomData,
			signer,
			signature,
			content: value
		})
	}
}

pub struct SignedVerified<A, B = ()> {
	
}

pub struct SignedUnverified<A, B = ()> {
	
}


// T designates the protocol the packet belongs to, () for generic or any

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
		let content: bytes::Bytes = value;
		Ok(Self {
			phantom_data: std::marker::PhantomData,
			content
		})
	}
}