use ed25519_dalek::Verifier as _;
use ed25519_dalek::Signer as _;

pub struct Ed25519Algorithm;

impl cryptography::AsymmetricSetLayout for Ed25519Algorithm {
	const PUBLIC_KEY_LEN: usize = 32;
	const SECRET_KEY_LEN: usize = 32;
	const SIGNATURE_LEN: usize = 64;
}

impl cryptography::AsymmetricKeyGenAlgorithm for Ed25519Algorithm {
	fn generate() -> cryptography::Result<cryptography::Pair> {
		let sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
		let vk = sk.verifying_key();
		let vk = vk.to_bytes().to_vec().into_boxed_slice();
		let sk = sk.to_bytes().to_vec().into_boxed_slice();
		
		Ok((vk, sk))
	}
}

impl cryptography::AsymmetricSignatureAlgorithm for Ed25519Algorithm {
	fn sign(sk: &cryptography::SecretKey, mg: &cryptography::Message) -> cryptography::Result<cryptography::Signature> {
		let sk: &[_; _] = sk.as_ref().try_into()?;
		let sk: ed25519_dalek::SigningKey = ed25519_dalek::SigningKey::from_bytes(sk);
		let sg: ed25519::Signature = sk.sign(mg);
		let sg: Box<_> = sg.to_vec().into_boxed_slice();
		
		Ok(sg)
	}
	
	fn verify(
        pk: &cryptography::PublicKey,
        mg: &cryptography::Message,
        sg: &cryptography::Signature
    ) -> cryptography::Result<bool> {
    	let vk: &[_; _] = pk.as_ref().try_into()?;
     	let vk: ed25519_dalek::VerifyingKey = ed25519_dalek::VerifyingKey::from_bytes(vk)?;
      	let sg: ed25519::Signature = ed25519_dalek::Signature::from_slice(sg)?;
       	let out: bool = vk.verify(mg, &sg).is_ok();
        
        Ok(out)
    }
}