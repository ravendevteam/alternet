use aes_gcm::KeyInit;
use pqcrypto_traits::kem::PublicKey as _;
use pqcrypto_traits::kem::SecretKey as _;
use pqcrypto_traits::kem::Ciphertext as _;
use pqcrypto_traits::kem::SharedSecret as _;
use pqcrypto_kyber::kyber1024;
use aes_gcm::aead;
use aes_gcm::aead::generic_array as aead_generic_array;
use rand_core::RngCore as _;
use super::*;

pub type PublicKey = [u8; 1568];
pub type SecretKey = [u8; 3168];
pub type Bytes = Box<[u8]>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidPublicKey,
    InvalidSecretKey,
    InvalidCipherText
}

pub struct Unset;

pub struct Kyber1024;

impl cryptography::AsymmetricKeyGenAlgorithm for Kyber1024 {
    type PublicKey = PublicKey;
    type SecretKey = SecretKey;
    type Rng = Unset;
    type Error = Error;

    fn generate(_: &mut Self::Rng) -> std::result::Result<(Self::PublicKey, Self::SecretKey), Self::Error> {
        let pair: (_, _) = kyber1024::keypair(); 
        let public_key: kyber1024::PublicKey = pair.0;
        let public_key: Self::PublicKey = public_key.as_bytes().into();
        let secret_key: kyber1024::SecretKey = pair.1;
        let secret_key: Self::SecretKey = secret_key.as_bytes().into();
        Ok((public_key, secret_key))
    }
}

impl cryptography::AsymmetricEncryptionAlgorithm for Kyber1024 {
    type PublicKey = PublicKey;
    type SecretKey = SecretKey;
    type Encrypted = Bytes;
    type Decrypted = Bytes;
    type Error = Error;

    fn encrypt(public_key: &Self::PublicKey, decrypted: &Self::Decrypted) -> std::result::Result<Self::Encrypted, Self::Error> {
        let public_key: kyber1024::PublicKey = kyber1024::PublicKey::from_bytes(public_key)
            .ok()
            .ok_or(Error::InvalidPublicKey)?;
        let pair: (_, _) = kyber1024::encapsulate(&public_key);
        let shared_secret: kyber1024::SharedSecret = pair.0;
        let cipher_text: kyber1024::Ciphertext = pair.1;
        let hkdf: hkdf::Hkdf<_> = hkdf::Hkdf::<sha2::Sha256>::new(None, shared_secret.as_bytes());
        let mut aead_key: [_; _] = [0u8; 32];
        hkdf.expand(b"Kyber1024-aes256gcm", &mut aead_key)
            .ok()
            .ok_or(Error::InvalidCipherText)?;
        let cipher_generic_array: aead_generic_array::GenericArray<u8, 32> = aead_generic_array::GenericArray::from_slice(&aead_key);
        let cipher = aes_gcm::Aes256Gcm::new(cipher_generic_array);
        let mut nonce: [_; _] = [0u8; 12];
        
    }

    fn decrypt(secret_key: &Self::SecretKey, encrypted: &Self::Encrypted) -> std::result::Result<Self::Decrypted, Self::Error> {
        
    }
}