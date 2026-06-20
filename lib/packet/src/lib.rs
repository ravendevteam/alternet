pub mod opcode;

pub struct IsUnknownProtocol;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct MarkedSignedVerified<A, B = IsUnknownProtocol> {
	signer: lib_cryptography::public_key::PublicKey<A>,
	signature: lib_cryptography::signature::Signature<A>,
	#[deref]
	#[deref_mut]
	content: Unsigned<B>
}

impl<A, B> TryFrom<MarkedSignedUnverified<A, B>> for MarkedSignedVerified<A, B>
where
	A: lib_cryptography::AsymmetricSignatureAlgorithm {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: MarkedSignedUnverified<A, B>) -> Result<Self, Self::Error> {
		let (signature, signer, unsigned) = value.into();
		let message: lib_bytes::NonEmpty = unsigned.into();
		let message: lib_cryptography::message::Message = message.into();
		if !A::verify(&signer, &message, &signature)? {
			return Err(<Box<dyn std::error::Error>>::from(String::from("invalid")))
		}
		let content: Unsigned<_> = message.into();
		Ok(Self {
			signer,
			signature,
			content
		})
	}
}

impl<A, B> TryFrom<(Unsigned<B>, &lib_cryptography::secret_key::SecretKey<A>)> for MarkedSignedVerified<A, B>
where
	A: lib_cryptography::AsymmetricKeyDerivationAlgorithm,
	A: lib_cryptography::AsymmetricSignatureAlgorithm {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: (Unsigned<B>, &lib_cryptography::secret_key::SecretKey<A>)) -> Result<Self, Self::Error> {
		let (unsigned, secret_key) = value;
		let message: lib_bytes::NonEmpty = unsigned.into();
		let message: lib_cryptography::message::Message = message.into();
		let signer: lib_cryptography::public_key::PublicKey<_> = secret_key.public_key();
		let signature: lib_cryptography::signature::Signature<_> = A::sign(&secret_key, &message)?;
		let content: Unsigned<_> = message.into();
		Ok(Self {
			signer,
			signature,
			content
		})
	}
}

impl<A, B> TryFrom<lib_bytes::NonEmpty> for MarkedSignedVerified<A, B> 
where
	A: lib_cryptography::AsymmetricSetLayout,
	A: lib_cryptography::AsymmetricSignatureAlgorithm {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: lib_bytes::NonEmpty) -> Result<Self, Self::Error> {
		let out: MarkedSignedUnverified<A, B> = value.try_into()?;
		let out: Self = out.try_into()?;
		Ok(out)
	}
}

impl<A, B> Into<(lib_cryptography::signature::Signature<A>, lib_cryptography::public_key::PublicKey<A>, Unsigned<B>)> for MarkedSignedVerified<A, B> {
	fn into(self) -> (lib_cryptography::signature::Signature<A>, lib_cryptography::public_key::PublicKey<A>, Unsigned<B>) {
		(
			self.signature,
			self.signer,
			self.content
		)
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct MarkedSignedUnverified<A, B = IsUnknownProtocol> {
	signer: lib_cryptography::public_key::PublicKey<A>,
	signature: lib_cryptography::signature::Signature<A>,
	#[deref]
	#[deref_mut]
	content: Unsigned<B>
}

impl<A, B> MarkedSignedUnverified<A, B>
where
	A: lib_cryptography::AsymmetricSignatureAlgorithm {
	pub fn verify(self) -> lib_kore::Result<MarkedSignedVerified<A, B>> {
		self.try_into()
	}
}

impl<A, B> TryFrom<lib_bytes::NonEmpty> for MarkedSignedUnverified<A, B>
where
	A: lib_cryptography::AsymmetricSetLayout {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: lib_bytes::NonEmpty) -> Result<Self, Self::Error> {
		let mut bytes: bytes::Bytes = value.into();
		let header_len: usize = A::PUBLIC_KEY_LEN + A::SIGNATURE_LEN;
		if bytes.len() <= header_len {
			return Err(<Box<dyn std::error::Error>>::from(String::from("packet too short; insufficient header length")))
		}
		let signer: bytes::Bytes = bytes.split_to(A::PUBLIC_KEY_LEN);
		let signer: lib_bytes::NonEmpty = signer.try_into()?;
		let signer: lib_cryptography::public_key::PublicKey<_> = signer.into();
		let signature: bytes::Bytes = bytes.split_to(A::SIGNATURE_LEN);
		let signature: lib_bytes::NonEmpty = signature.try_into()?;
		let signature: lib_cryptography::signature::Signature<_> = signature.into();
		let content: lib_bytes::NonEmpty = bytes.try_into()?;
		let content: Unsigned<_> = content.into();
		Ok(Self {
			signer,
			signature,
			content
		})
	}
}

impl<A, B> Into<(lib_cryptography::signature::Signature<A>, lib_cryptography::public_key::PublicKey<A>, Unsigned<B>)> for  MarkedSignedUnverified<A, B> {
	fn into(self) -> (lib_cryptography::signature::Signature<A>, lib_cryptography::public_key::PublicKey<A>, Unsigned<B>) {
		(
			self.signature,
			self.signer,
			self.content
		)
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct SignedVerified<A, B = IsUnknownProtocol> {
	signature: lib_cryptography::signature::Signature<A>,
	#[deref]
	#[deref_mut]
	content: Unsigned<B>
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct SignedUnverified<A, B = IsUnknownProtocol> {
	signature: lib_cryptography::signature::Signature<A>,
	#[deref]
	#[deref_mut]
	content: Unsigned<B>
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Unsigned<T = IsUnknownProtocol> {
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
