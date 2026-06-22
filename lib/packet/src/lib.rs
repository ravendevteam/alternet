pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct IsUnsetSigner;
pub struct IsUnsetSignature;
pub struct IsUnsetLayout;
pub struct IsUnsetAlgorithm;
pub struct IsUnsetProtocol;
pub struct IsMarkedSignedVerified;
pub struct IsMarkedSignedUnverified;
pub struct IsSignedVerified;
pub struct IsSignedUnverified;
pub struct IsUnsigned;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Packet<A = IsUnsigned, B = IsUnsetLayout, C = IsUnsetAlgorithm, D = IsUnsetProtocol, E = IsUnsetSigner, F = IsUnsetSignature> {
	phantom_data: std::marker::PhantomData<(A, C, D)>,
	#[deref]
	#[deref_mut]
	content: B,
	signer: E,
	signature: F
}

pub type Unsigned<A = IsUnsetLayout, B = IsUnsetProtocol> = Packet<IsUnsigned, A, IsUnsetAlgorithm, B>;

impl<A, B> Unsigned<A, B> {
	pub fn content(&self) -> &A {
		&self.content
	}
}

impl<A, B> TryFrom<lib_cryptography::message::Message> for Unsigned<A, B> 
where
	A: TryFrom<lib_bytes::NonEmpty, Error = Box<dyn std::error::Error>> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: lib_cryptography::message::Message) -> std::result::Result<Self, Self::Error> {
		let out: lib_bytes::NonEmpty = value.into();
		let out: A = out.try_into()?;
		let out: Self = Self {
			phantom_data: std::marker::PhantomData,
			content: out,
			signer: IsUnsetSigner,
			signature: IsUnsetSignature
		};
		Ok(out)
	}
}

impl<A, B> TryFrom<lib_bytes::NonEmpty> for Unsigned<A, B> 
where
	A: TryFrom<lib_bytes::NonEmpty, Error = Box<dyn std::error::Error>> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: lib_bytes::NonEmpty) -> std::result::Result<Self, Self::Error> {
		let out: A = value.try_into()?;
		let out: Self = Self {
			phantom_data: std::marker::PhantomData,
			content: out,
			signer: IsUnsetSigner,
			signature: IsUnsetSignature
		};
		Ok(out)
	}
}

impl<A, B> TryInto<lib_bytes::NonEmpty> for Unsigned<A, B>
where
	A: TryInto<lib_bytes::NonEmpty, Error = Box<dyn std::error::Error>> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_into(self) -> std::result::Result<lib_bytes::NonEmpty, Self::Error> {
		let out: A = self.content.into();
		let out: lib_bytes::NonEmpty = out.try_into()?;
		Ok(out)
	}
}

pub type MarkedSignedVerified<A = IsUnsetLayout, B = IsUnsetAlgorithm, C = IsUnsetProtocol> = Packet<IsMarkedSignedVerified, A, B, C, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>>;

impl<A, B, C> MarkedSignedVerified<A, B, C> {
	pub fn content(&self) -> &A {
		&self.content
	}

	pub fn signer(&self) -> &lib_cryptography::public_key::PublicKey<B> {
		&self.signer
	}

	pub fn signature(&self) -> &lib_cryptography::signature::Signature<B> {
		&self.signature
	}
}

impl<A, B, C> TryFrom<MarkedSignedUnverified<A, B, C>> for MarkedSignedVerified<A, B, C>
where
	A: Clone,
	A: Into<lib_bytes::NonEmpty>,
	B: lib_cryptography::AsymmetricSignatureAlgorithm {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: MarkedSignedUnverified<A, B, C>) -> std::result::Result<Self, Self::Error> {
		let (content, signer, signature) = value.into();
		let message: lib_bytes::NonEmpty = content.to_owned().try_into()?;
		let message: lib_cryptography::message::Message = message.into();
		if !B::verify(&signer, &message, &signature)? {
			return Err(<Box<dyn std::error::Error>>::from(String::from("invalid")))
		}
		let out: Self = Self {
			phantom_data: std::marker::PhantomData,
			content,
			signer,
			signature
		};
		Ok(out)
	}
}

impl<A, B, C> TryFrom<lib_bytes::NonEmpty> for MarkedSignedVerified<A, B, C>
where
	A: Clone,
	A: TryFrom<lib_bytes::NonEmpty, Error = Box<dyn std::error::Error>>,
	B: lib_cryptography::AsymmetricSetLayout,
	B: lib_cryptography::AsymmetricSignatureAlgorithm {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: lib_bytes::NonEmpty) -> std::result::Result<Self, Self::Error> {
		let mut bytes: bytes::Bytes = value.into();
		let header_len: usize = B::PUBLIC_KEY_LEN + B::SIGNATURE_LEN;
		if bytes.len() <= header_len {
			return Err(<Box<dyn std::error::Error>>::from(String::from("packet too short; insufficient header length")))
		}
		let signer: bytes::Bytes = bytes.split_to(B::PUBLIC_KEY_LEN);
		let signer: lib_bytes::NonEmpty = signer.try_into()?;
		let signer: lib_cryptography::public_key::PublicKey<B> = signer.into();
		let signature: bytes::Bytes = bytes.split_to(B::SIGNATURE_LEN);
		let signature: lib_bytes::NonEmpty = signature.try_into()?;
		let signature: lib_cryptography::signature::Signature<B> = signature.into();
		let content: lib_bytes::NonEmpty = bytes.try_into()?;
		let content: lib_cryptography::message::Message = content.into();
		if !B::verify(&signer, &content, &signature)? {
			return Err(<Box<dyn std::error::Error>>::from(String::from("invalid")))
		}
		let content: Unsigned<A, C> = content.try_into()?;
		let content: A = content.content().to_owned();
		Ok(Self {
			phantom_data: std::marker::PhantomData,
			content,
			signer,
			signature
		})
	}
}

impl<A, B, C> Into<(A, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>)> for Packet<IsMarkedSignedVerified, A, B, C, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>> {
	fn into(self) -> (A, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>) {
		(
			self.content,
			self.signer,
			self.signature
		)
	}
}

pub type MarkedSignedUnverified<A = IsUnsetLayout, B = IsUnsetAlgorithm, C = IsUnsetProtocol> = Packet<IsMarkedSignedUnverified, A, B, C, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>>;

impl<A, B, C> MarkedSignedUnverified<A, B, C> 
where
	A: Clone,
	A: Into<lib_bytes::NonEmpty>,
	B: lib_cryptography::AsymmetricSignatureAlgorithm {
	pub fn verify(self) -> Result<MarkedSignedVerified<A, B, C>> {
		self.try_into()
	}
}

impl<A, B, C> TryFrom<lib_bytes::NonEmpty> for MarkedSignedUnverified<A, B, C>
where
	A: Clone,
	A: TryFrom<lib_bytes::NonEmpty, Error = Box<dyn std::error::Error>>,
	B: lib_cryptography::AsymmetricSetLayout {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: lib_bytes::NonEmpty) -> std::result::Result<Self, Self::Error> {
		let mut bytes: bytes::Bytes = value.into();
		let header_len: usize = B::PUBLIC_KEY_LEN + B::SIGNATURE_LEN;
		if bytes.len() <= header_len {
			return Err(<Box<dyn std::error::Error>>::from(String::from("packet too short; insufficient header length")))
		}
		let signer: bytes::Bytes = bytes.split_to(B::PUBLIC_KEY_LEN);
		let signer: lib_bytes::NonEmpty = signer.try_into()?;
		let signer: lib_cryptography::public_key::PublicKey<B> = signer.into();
		let signature: bytes::Bytes = bytes.split_to(B::SIGNATURE_LEN);
		let signature: lib_bytes::NonEmpty = signature.try_into()?;
		let signature: lib_cryptography::signature::Signature<B> = signature.into();
		let content: lib_bytes::NonEmpty = bytes.try_into()?;
		let content: A = content.try_into()?;
		Ok(Self {
			phantom_data: std::marker::PhantomData,
			content: content.try_into()?,
			signer,
			signature
		})
	}
}

impl<A, B, C> Into<(A, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>)> for MarkedSignedUnverified<A, B, C> {
	fn into(self) -> (A, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>) {
		(
			self.content,
			self.signer,
			self.signature
		)
	}
}