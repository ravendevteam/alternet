use super::*;
use ed25519_dalek::Signer as _;
use ed25519::signature::Keypair as _;

pub mod commitment {
	
}

pub mod multi_party {
	use super::*;
		
	pub struct CompleteVerified(Vec<u8>);
	
	impl TryFrom<Partial> for CompleteVerified {
		type Error = Box<dyn std::error::Error>;
		
		fn try_from(value: Partial) -> std::result::Result<Self, Self::Error> {
			
		}
	}
	
	// all parties have signed
	pub struct CompleteUnverified(Vec<u8>);
	
	pub struct CompleteVerified {
		signers: Vec<PublicKey>,
		content: Vec<u8>
	}
	
	#[derive(Debug)]
	#[derive(Clone)]
	#[derive(PartialEq)]
	#[derive(Eq)]
	pub struct Partial {
		mg: Vec<u8>,
		pk_to_sg: std::collections::HashMap<PublicKey, Signature>,
		signers: Vec<PublicKey>
	}
	
	impl Partial {
		pub fn sign(self, pk: PublicKey, sg: Signature) -> Result<Or<Self, CompleteVerified>> {
			sg.verify(mg, &pk)?;
			
			self.pk_to_sg.insert(pk, sg);
			Ok(Or::Lhs(self))
		}
	}
	
	impl TryFrom<Vec<u8>> for Partial {
		type Error = Box<dyn std::error::Error>;
		
		fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
			
		}
	}
	
	pub struct Unsigned(Vec<u8>);
}

pub mod single_party {
	use super::*;
	
	#[derive(Debug)]
	#[derive(Clone)]
	#[derive(PartialEq)]
	#[derive(Eq)]
	#[derive(derive_more::Deref)]
	#[derive(derive_more::DerefMut)]
	pub struct SignedVerified {
		pk: PublicKey,
		#[deref]
		#[deref_mut]
		content: Vec<u8>
	}
	
	impl SignedVerified {
		pub fn pk(&self) -> &PublicKey {
			&self.pk
		}
		
		pub fn content(&self) -> &[u8] {
			&self.content
		}
	}
	
	impl Unpack<(PublicKey, Unsigned)> for SignedVerified {
		fn unpack(self) -> (PublicKey, Unsigned) {
			let mg: Unsigned = self.content.into();
			(self.pk, mg)
		}
	}
	
	impl TryFrom<SignedUnverified> for SignedVerified {
		type Error = Box<dyn std::error::Error>;
		
		fn try_from(value: SignedUnverified) -> std::result::Result<Self, Self::Error> {
			let (sg, pk, mg) = value.unpack();
	
			sg.verify(&mg, &pk)?;
			
			let Unsigned(content) = mg;
			Ok(Self {
				pk: pk.to_owned(),
				content
			})
		}
	}
	
	#[derive(Debug)]
	#[derive(Clone)]
	#[derive(PartialEq)]
	#[derive(Eq)]
	#[derive(serde::Serialize)]
	#[derive(serde::Deserialize)]
	#[derive(derive_more::Deref)]
	#[derive(derive_more::DerefMut)]
	#[serde(try_from = "Vec<u8>")]
	pub struct SignedUnverified(Vec<u8>);
	
	impl SignedUnverified {
		pub fn verify(self) -> Result<SignedVerified> {
			self.try_into()
		}
	}
	
	impl Unpack<(Signature, PublicKey, Unsigned)> for SignedUnverified {
		fn unpack(self) -> (Signature, PublicKey, Unsigned) {
			let Self(mut bytes) = self;				
			let mut mgpk: Vec<_> = bytes.split_off(64);
			let mg: Vec<_> = mgpk.split_off(32);
			let mg: Unsigned = mg.try_into().expect("unsigned content slice is guaranteed to be non-empty via constructors");
			let pk: [_; _] = mgpk.as_slice().try_into().expect("public key state corruption; must equal 32 bytes");
			let pk: PublicKey = pk.into();
			let sg: [_; _] = bytes.as_slice().try_into().expect("signature state corruption; remaining bytes must equal 64; guaranteed via constructors");
			let sg: Signature = sg.into();
			(sg, pk, mg)
		}
	}
	
	impl From<(&Unsigned, &SecretKey)> for SignedUnverified {
		fn from(value: (&Unsigned, &SecretKey)) -> Self {
			let (Unsigned(mg), SecretKey(sk)) = value;
			let sk: ed25519_dalek::SigningKey = ed25519_dalek::SigningKey::from_bytes(&sk);
			let pk: [_; _] = sk.verifying_key().to_bytes();
			let sg: ed25519_dalek::Signature = sk.sign(&mg);
			let sg: [_; _] = sg.to_bytes();
			let capacity: usize = 64 + 32 + mg.len();
			
			let mut bytes: Vec<_> = Vec::with_capacity(capacity);
			bytes.extend_from_slice(&sg);
			bytes.extend_from_slice(&pk);
			bytes.extend_from_slice(&mg);
			
			Self(bytes)
		}
	}
	
	impl TryFrom<Vec<u8>> for SignedUnverified {
		type Error = Box<dyn std::error::Error>;
		
		fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
			if value.len() <= 96 {
				return Err(<Box<dyn std::error::Error>>::from(String::from("signed message packet is too short")))
			}
			let content: &[_] = &value[96..];
			if content.is_empty() {
				return Err(<Box<dyn std::error::Error>>::from(String::from("signed message content cannot be empty")))
			}
			Ok(Self(value))
		}
	}
	
	#[derive(Debug)]
	#[derive(Clone)]
	#[derive(PartialEq)]
	#[derive(Eq)]
	#[derive(serde::Serialize)]
	#[derive(serde::Deserialize)]
	#[derive(derive_more::Deref)]
	#[derive(derive_more::DerefMut)]
	#[serde(try_from = "Vec<u8>")]
	pub struct Unsigned(Vec<u8>);
	
	impl Unsigned {
		pub fn sign(&self, sk: &SecretKey) -> SignedUnverified {
			(self, sk).into()
		}
	}
	
	impl TryFrom<Vec<u8>> for Unsigned {
		type Error = Box<dyn std::error::Error>;
		
		fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
			if value.len() == 0 {
				return Err(<Box<dyn std::error::Error>>::from(String::from("message too short")))
			}
			Ok(Self(value))
		}
	}	
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct PublicKey([u8; 32]);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct SecretKey([u8; 32]);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct Signature([u8; 64]);

impl Signature {
	pub fn verify(&self, mg: &[u8], pk: &PublicKey) -> Result {
		let pk: ed25519_dalek::VerifyingKey = ed25519_dalek::VerifyingKey::from_bytes(&pk.0)?;
		let sg: ed25519_dalek::Signature = ed25519_dalek::Signature::from_bytes(&self.0);
		
		pk.verify(mg, &sg)
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct Pair((PublicKey, SecretKey));

impl Default for Pair {
	fn default() -> Self {
		let sk: ed25519_dalek::SigningKey = ed25519_dalek::SigningKey::generate(&mut rand_core::OsRng);
		let vk: ed25519_dalek::VerifyingKey = sk.verifying_key();
		let vk: [_; _] = vk.to_bytes();
		let pk: PublicKey = PublicKey::from(vk);
		let sk: [_; _] = sk.to_bytes();
		let sk: SecretKey = SecretKey::from(sk);
		
		Self::from((pk, sk))
	}
}

impl From<SecretKey> for Pair {
	fn from(value: SecretKey) -> Self {
		let SecretKey(bytes) = value;
		let sk: ed25519_dalek::SigningKey = ed25519_dalek::SigningKey::from_bytes(&bytes);
		let vk: ed25519_dalek::VerifyingKey = sk.verifying_key();
		let vk: [_; _] = vk.to_bytes();
		let pk: PublicKey = PublicKey::from(vk);
		let sk: [_; _] = sk.to_bytes();
		let sk: SecretKey = SecretKey::from(sk);
		
		Self::from((pk, sk))
	}
}