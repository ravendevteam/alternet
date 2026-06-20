use super::*;

pub struct IsFromMarkedSignedVerified;
pub struct IsFromMarkedSignedUnverified;
pub struct IsFromSignedVerified;
pub struct IsFromSignedUnverified;
pub struct IsFromUnsigned;

#[derive(Debug)]
#[derive(Clone)]
pub struct Call<A, B, C = IsUnknownProtocol, D = IsFromUnsigned, E = (), F = ()> {
	phantom_data: std::marker::PhantomData<(B, C, D)>,
	pkt: A,
	signer: E,
	signature: F
}

impl<A, B, C> TryFrom<lib_bytes::NonEmpty> for Call<A, B, C, IsFromMarkedSignedVerified, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>> 
where
	A: TryFrom<Unsigned<C>, Error = Box<dyn std::error::Error>>,
	B: lib_cryptography::AsymmetricSetLayout,
	B: lib_cryptography::AsymmetricSignatureAlgorithm {
	type Error = Box<dyn std::error::Error>;
	
	fn try_from(value: lib_bytes::NonEmpty) -> Result<Self, Self::Error> {
		let out: MarkedSignedUnverified<B, C> = value.try_into()?;
		let out: MarkedSignedVerified<_, _> = out.try_into()?;
		let (signature, signer, message) = out.into();
		Ok(Self {
			phantom_data: std::marker::PhantomData,
			pkt: message.try_into()?,
			signer,
			signature
		})
	}
}

impl<A, B, C> Into<(A, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>)> for Call<A, B, C, IsFromMarkedSignedVerified, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>> {
	fn into(self) -> (A, lib_cryptography::public_key::PublicKey<B>, lib_cryptography::signature::Signature<B>) {
		(
			self.pkt,
			self.signer,
			self.signature
		)
	}
}