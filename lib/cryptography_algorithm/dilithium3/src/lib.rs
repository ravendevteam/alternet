use pqcrypto_traits::sign::DetachedSignature as _;
use pqcrypto_traits::sign::PublicKey as _;
use pqcrypto_traits::sign::SecretKey as _;
use pqcrypto_dilithium::dilithium3;

pub type Bytes = Box<[u8]>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[derive(thiserror::Error)]
pub enum Error {
    #[error("invalid secret key")]
    InvalidSecretKey,

    #[error("invalid signature")]
    InvalidSignature
}

pub struct Dilithium3;

impl cryptography::AsymmetricKeyGenAlgorithm for Dilithium3 {
    fn generate() -> cryptography::Result<cryptography::Pair> {
        let pair: (_, _) = dilithium3::keypair();
        let pk = pair.0;
        let pk: Bytes = pk.as_bytes().to_vec().into();
        let sk = pair.1;
        let sk: Bytes = sk.as_bytes().to_vec().into();
        Ok((pk, sk))
    }
}

impl cryptography::AsymmetricSignatureAlgorithm for Dilithium3 {
    fn sign(sk: &cryptography::SecretKey, msg: &cryptography::Message) -> cryptography::Result<cryptography::Signature> {
        let sk: dilithium3::SecretKey = dilithium3::SecretKey::from_bytes(sk)
            .ok()
            .ok_or(Error::InvalidSecretKey)?;
        let sig: dilithium3::DetachedSignature = dilithium3::detached_sign(msg, &sk);
        let sig: Bytes = sig.as_bytes().to_owned().into();
        Ok(sig)
    }

    fn verify(
        pk: &cryptography::PublicKey,
        msg: &cryptography::Message,
        sig: &cryptography::Signature
    ) -> cryptography::Result<bool> {
        let pk: dilithium3::PublicKey = dilithium3::PublicKey::from_bytes(pk)
            .ok()
            .ok_or(Error::InvalidSecretKey)?;
        let sig: dilithium3::DetachedSignature = dilithium3::DetachedSignature::from_bytes(sig)
            .ok()
            .ok_or(Error::InvalidSignature)?;
        let ret: bool = dilithium3::verify_detached_signature(&sig, msg, &pk).is_ok();
        Ok(ret)
    }
}