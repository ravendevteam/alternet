use ed25519_dalek::Verifier as _;
use ed25519_dalek::Signer as _;

const PUBLIC_KEY_LEN: usize = 32;
const SECRET_KEY_LEN: usize = 32;
const SIGNATURE_LEN: usize = 64;

#[derive(Debug)]
pub struct Ed25519Algorithm;

impl cryptography::Algorithm for Ed25519Algorithm {}

impl cryptography::AsymmetricSetLayout for Ed25519Algorithm {
	const PUBLIC_KEY_LEN: usize = PUBLIC_KEY_LEN;
	const SECRET_KEY_LEN: usize = SECRET_KEY_LEN;
	const SIGNATURE_LEN: usize = SIGNATURE_LEN;
}

impl cryptography::AsymmetricKeyGenAlgorithm for Ed25519Algorithm {
	fn generate() -> cryptography::Result<cryptography::pair::Pair<Self>> {
		let signing_key: ed25519_dalek::SigningKey = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
		let verifying_key: ed25519_dalek::VerifyingKey = signing_key.verifying_key();
		let verifying_key: &[_; _] = verifying_key.as_bytes();
		let signing_key: [_; _] = signing_key.to_bytes();
		let signing_key: &[_] = signing_key.as_slice();
		let secret_key: bytes::Bytes = bytes::Bytes::copy_from_slice(signing_key);
		let secret_key: cryptography::bytes::Bytes = secret_key.try_into()?;
		let secret_key: cryptography::secret_key::SecretKey<_> = secret_key.into();
		let public_key: bytes::Bytes = bytes::Bytes::copy_from_slice(verifying_key);
		let public_key: cryptography::bytes::Bytes = public_key.try_into()?;
		let public_key: cryptography::public_key::PublicKey<_> = public_key.into();
		Ok(cryptography::pair::Pair::from((public_key, secret_key)))
	}
}

impl cryptography::AsymmetricSignatureAlgorithm for Ed25519Algorithm {
	fn sign(secret_key: &cryptography::secret_key::SecretKey<Self>, message: &cryptography::message::Message) -> cryptography::Result<cryptography::signature::Signature<Self>> {
		let message: &[_] = message.as_ref();
		let secret_key: [u8; SECRET_KEY_LEN] = secret_key.as_ref().try_into()?;
		let signing_key: ed25519_dalek::SigningKey = ed25519_dalek::SigningKey::from_bytes(&secret_key);
		let out: ed25519::Signature = signing_key.sign(message);
		let out: [_; _] = out.to_bytes();
		let out: bytes::Bytes = bytes::Bytes::copy_from_slice(&out);
		let out: cryptography::bytes::Bytes = out.try_into()?;
		let out: cryptography::signature::Signature<Self> = out.into();
		Ok(out)
	}
	
	fn verify(public_key: &cryptography::public_key::PublicKey<Self>, message: &cryptography::message::Message, signature: &cryptography::signature::Signature<Self>) -> cryptography::Result<bool> {
		let message: &[_] = message.as_ref();
		let public_key: [u8; PUBLIC_KEY_LEN] = public_key.as_ref().try_into()?;
		let verifying_key: ed25519_dalek::VerifyingKey = ed25519_dalek::VerifyingKey::from_bytes(&public_key)?;
		let signature: &[_; _] = signature.as_ref().try_into()?;
		let signature: ed25519::Signature = ed25519_dalek::Signature::from_bytes(signature);
		let out: bool = verifying_key.verify(message, &signature).is_ok();
		Ok(out)
	}
}