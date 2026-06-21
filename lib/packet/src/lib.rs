pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

// might need to be Clone
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
pub struct Packet<A = IsUnsigned, B = IsUnsetLayout, C = IsUnsetAlgorithm, D = IsUnsetProtocol, E = IsUnsetSigner, F = IsUnsetSignature> {
	phantom_data: std::marker::PhantomData<(A, C, D)>,
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

impl<A, B> Into<A> for Unsigned<A, B> {
	fn into(self) -> A {
		self.content
	}
}

impl<A, B> TryInto<lib_bytes::NonEmpty> for Unsigned<A, B>
where
	A: TryInto<lib_bytes::NonEmpty, Error = Box<dyn std::error::Error>> {
	type Error = Box<dyn std::error::Error>;
	
	fn try_into(self) -> std::result::Result<lib_bytes::NonEmpty, Self::Error> {
		let out: A = self.into();
		let out: lib_bytes::NonEmpty = out.try_into()?;
		Ok(out)
	}
}

pub type MarkedSignedVerified<A = IsUnsetLayout, B = IsUnsetAlgorithm, C = IsUnsetProtocol> = Packet<IsMarkedSignedVerified, A, B, C, lib_cryptography::public_key::PublicKey<C>, lib_cryptography::signature::Signature<C>>;

impl<A, B, C> MarkedSignedVerified<A, B, C> {
	pub fn content(&self) -> &A {
		&self.content
	}

	pub fn signer(&self) -> &lib_cryptography::public_key::PublicKey<C> {
		&self.signer
	}

	pub fn signature(&self) -> &lib_cryptography::signature::Signature<C> {
		&self.signature
	}
}

impl<A, B, C> TryFrom<MarkedSignedUnverified<A, B, C>> for MarkedSignedVerified<A, B, C>
where
	B: lib_cryptography::AsymmetricSignatureAlgorithm {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: Packet<IsMarkedSignedUnverified, A, B, C>) -> std::result::Result<Self, Self::Error> {
		let (content, signer, signature) = value.into();

	}
}

impl<A, B, C> TryFrom<lib_bytes::NonEmpty> for Packet<IsMarkedSignedVerified, A, B, C, lib_cryptography::public_key::PublicKey<C>, lib_cryptography::signature::Signature<C>>
where
	B: TryFrom<Unsigned<D>, Error = Box<dyn std::error::Error>>,
	C: lib_cryptography::AsymmetricSetLayout,
	C: lib_cryptography::AsymmetricSignatureAlgorithm {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: lib_bytes::NonEmpty) -> Result<Self, Self::Error> {
		let mut bytes: bytes::Bytes = value.into();
		let header_len: usize = C::PUBLIC_KEY_LEN + C::SIGNATURE_LEN;
		if bytes.len() <= header_len {
			return Err(<Box<dyn std::error::Error>>::from(String::from("packet too short; insufficient header length")))
		}
		let signer: bytes::Bytes = bytes.split_to(B::PUBLIC_KEY_LEN);
		let signer: lib_bytes::NonEmpty = signer.try_into()?;
		let signer: lib_cryptography::public_key::PublicKey<C> = signer.into();
		let signature: bytes::Bytes = bytes.split_to(B::SIGNATURE_LEN);
		let signature: lib_bytes::NonEmpty = signature.try_into()?;
		let signature: lib_cryptography::signature::Signature<C> = signature.into();
		let content: lib_bytes::NonEmpty = bytes.try_into()?;
		let content: lib_cryptography::message::Message = content.into();
		if !A::verify(&signer, &content, &signature)? {
			return Err(<Box<dyn std::error::Error>>::from(String::from("invalid")))
		}
		let content: Unsigned<D> = content.into();
		Ok(Self {
			phantom_data: std::marker::PhantomData,
			content: content.try_into()?,
			signer,
			signature
		})
	}
}

pub type MarkedSignedUnverified<A = IsUnsetLayout, B = IsUnsetAlgorithm, C = IsUnsetProtocol> = Packet<IsMarkedSignedUnverified, A, B, C, lib_cryptography::public_key::PublicKey<C>, lib_cryptography::signature::Signature<C>>;

impl<A, B, C> MarkedSignedUnverified<A, B, C> 
where
	B: lib_cryptography::AsymmetricSignatureAlgorithm {
	pub fn verify(self) -> Result<MarkedSignedVerified<A, B, C>> {
		self.try_into()
	}
}

















impl<A, B, C> Into<(A, lib_cryptography::public_key::PublicKey<C>, lib_cryptography::signature::Signature<C>)> for Packet<IsMarkedSignedVerified, A, B, C, lib_cryptography::public_key::PublicKey<C>, lib_cryptography::signature::Signature<C>> {
	fn into(self) -> (A, lib_cryptography::public_key::PublicKey<C>, lib_cryptography::signature::Signature<C>) {
		(
			self.content,
			self.signer,
			self.signature
		)
	}
}

impl<A, B, C> Into<(A, lib_cryptography::public_key::PublicKey<C>, lib_cryptography::signature::Signature<C>)> for Packet<IsMarkedSignedUnverified, A, B, C, lib_cryptography::public_key::PublicKey<C>, lib_cryptography::signature::Signature<C>> {
	fn into(self) -> (A, lib_cryptography::public_key::PublicKey<C>, lib_cryptography::signature::Signature<C>) {
		(
			self.content,
			self.signer,
			self.signature
		)
	}
}





impl<A, B, C> TryFrom<lib_bytes::NonEmpty> for Packet<A, B, C, IsMarkedSignedUnverified, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>>
where
	A: TryFrom<Unsigned<C>, Error = Box<dyn std::error::Error>>,
	B: lib_cryptography::AsymmetricSetLayout {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: lib_bytes::NonEmpty) -> Result<Self, Self::Error> {
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
		let content: Unsigned<C> = content.into();
		Ok(Self {
			phantom_data: std::marker::PhantomData,
			content: content.try_into()?,
			signer,
			signature
		})
	}
}

impl<A, B, C> TryFrom<lib_bytes::NonEmpty> for Packet<A, B, C>
where
	A: TryFrom<Unsigned<C>, Error = Box<dyn std::error::Error>> {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: lib_bytes::NonEmpty) -> Result<Self, Self::Error> {
		let content: lib_bytes::NonEmpty = value;
		let content: Unsigned<_> = content.into();
		Ok(Self {
			phantom_data: std::marker::PhantomData,
			content: content.try_into()?,
			signer: IsUnsetSigner,
			signature: IsUnsetSigner
		})
	}
}

impl<A, B, C> Into<(A, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>)> for Packet<A, B, C, IsMarkedSignedVerified, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>> {
	fn into(self) -> (A, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>) {
		(
			self.content,
			self.signer,
			self.signature
		)
	}
}

impl<A, B, C> TryInto<lib_bytes::NonEmpty> for Packet<A, B, C, IsMarkedSignedVerified, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>>
where
	A: TryInto<Unsigned<C>, Error = Box<dyn std::error::Error>> {
	type Error = Box<dyn std::error::Error>;

	fn try_into(self) -> Result<lib_bytes::NonEmpty, Self::Error> {
		let unsigned: Unsigned<_> = self.content.try_into()?;
		let out: MarkedSignedUnverified<B, C> = (self.signer, self.signature, unsigned).into();
		let out: MarkedSignedVerified<_, _> = out.try_into()?;
		let out: lib_bytes::NonEmpty = out.into();
		Ok(out)
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct MarkedSignedVerified<A, B = IsUnsetProtocol> {
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
struct MarkedSignedUnverified<A = IsUnsetLayout, B = IsUnsetAlgorithm, C = IsUnsetProtocol> {
	phantom_data: std::marker::PhantomData<C>,
	signer: lib_cryptography::public_key::PublicKey<B>,
	signature: lib_cryptography::signature::Signature<B>,
	#[deref]
	#[deref_mut]
	content: A
}

impl<A, B, C> MarkedSignedUnverified<A, B>
where
	A: lib_cryptography::AsymmetricSignatureAlgorithm {
	pub fn verify(self) -> lib_kore::Result<MarkedSignedVerified<A, B>> {
		self.try_into()
	}
}

impl<A, B> From<(lib_cryptography::public_key::PublicKey<A>, lib_cryptography::signature::Signature<A>)> for MarkedSignedUnverified<A, B> {
	fn from(value: (lib_cryptography::public_key::PublicKey<A>, lib_cryptography::signature::Signature<A>, Unsigned<B>)) -> Self {
		let (signer, signature, content) = value;
		Self {
			signer,
			signature,
			content
		}
	}
}

impl<A, B> TryFrom<(Unsigned<B>, lib_cryptography::public_key::PublicKey<A>, lib_cryptography::signature::Signature<A>)> for MarkedSignedUnverified<A, B>
where
	A TryFrom<Unsigned<B>> {
	type Error = Box<dyn std::error::Error>;

	fn try_from(value: (Unsigned<B>, lib_cryptography::public_key::PublicKey<A>, lib_cryptography::signature::Signature<A>)) -> Result<Self, Self::Error> {
		let (content, signer, signature) = value;


		Ok(Self {
			signer,
			signature,
			content: content.try_into()?
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
struct SignedVerified<A, B = IsUnsetProtocol> {
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
struct SignedUnverified<A, B = IsUnsetProtocol> {
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
