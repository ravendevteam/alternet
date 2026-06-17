pub mod encrypted;
pub mod key;
pub mod pair;
pub mod public_key;
pub mod secret_key;
pub mod signature;
pub mod signed;

pub type Bytes = Box<[u8]>;
pub type Key = Bytes;
pub type PublicKey = Bytes;
pub type SecretKey = Bytes;
pub type Signature = Bytes;
pub type Message = Bytes;
pub type Pair = (PublicKey, SecretKey);

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait AsymmetricKeyGenAlgorithm {
    fn generate() -> Result<Pair>;
}

pub trait AsymmetricSignatureAlgorithm {
    fn sign(sk: &SecretKey, msg: &Message) -> Result<Signature>;
    fn verify(
        pk: &PublicKey,
        msg: &Message,
        sig: &Signature
    ) -> Result<bool>;
}

pub trait AsymmetricEncryptionAlgorithm {
    fn encrypt(pk: &PublicKey, bytes: &Bytes) -> Result<Bytes>;
    fn decrypt(sk: &SecretKey, bytes: &Bytes) -> Result<Bytes>;
}

pub trait SymmetricKeyGenAlgorithm {
    fn generate() -> Result<Key>;
}

pub trait SymmetricEncryptionAlgorithm {
    fn encrypt(key: &Key, bytes: &Bytes) -> Result<Bytes>;
    fn decrypt(key: &Key, bytes: &Bytes) -> Result<Bytes>;
}