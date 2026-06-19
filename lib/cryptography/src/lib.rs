pub mod encrypted;
pub mod key;
pub mod message;
pub mod pair;
pub mod public_key;
pub mod secret_key;
pub mod signature;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Algorithm 
where
	Self: Sized {}

pub trait AsymmetricSetLayout {
	const PUBLIC_KEY_LEN: usize;
	const SECRET_KEY_LEN: usize;
	const SIGNATURE_LEN: usize;
}

pub trait AsymmetricKeyDerivationAlgorithm 
where
	Self: Algorithm {
	fn public_key(secret_key: &secret_key::SecretKey<Self>) -> public_key::PublicKey<Self>;
}

pub trait AsymmetricKeyGenAlgorithm 
where
	Self: Algorithm {
    fn generate() -> Result<pair::Pair<Self>>;
}

pub trait AsymmetricSignatureAlgorithm
where
	Self: Algorithm {
    fn sign(secret_key: &secret_key::SecretKey<Self>, message: &message::Message) -> Result<signature::Signature<Self>>;
    fn verify(public_key: &public_key::PublicKey<Self>, message: &message::Message, signature: &signature::Signature<Self>) -> Result<bool>;
}

pub trait AsymmetricEncryptionAlgorithm 
where
	Self: Algorithm {
    fn encrypt(public_key: &public_key::PublicKey<Self>, message: &message::Message) -> Result<encrypted::Encrypted<Self>>;
    fn decrypt(secret_key: &secret_key::SecretKey<Self>, message: &encrypted::Encrypted<Self>) -> Result<message::Message>;
}

pub trait SymmetricKeyGenAlgorithm 
where
	Self: Algorithm {
    fn generate() -> Result<key::Key<Self>>;
}

pub trait SymmetricEncryptionAlgorithm 
where
	Self: Algorithm {
    fn encrypt(key: &key::Key<Self>, message: &message::Message) -> Result<encrypted::Encrypted<Self>>;
    fn decrypt(key: &key::Key<Self>, message: &encrypted::Encrypted<Self>) -> Result<message::Message>;
}