use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Pair<T> {
    #[serde(skip)]
    phantom_data: std::marker::PhantomData<T>,
    pub pk: public_key::PublicKey<T>,
    pub sk: secret_key::SecretKey<T>
}

impl<T> Pair<T> {
    pub fn from_existing(pk: public_key::PublicKey<T>, sk: secret_key::SecretKey<T>) -> Self {
        let phantom_data: std::marker::PhantomData<_> = std::marker::PhantomData;
        Self {
            phantom_data,
            pk,
            sk
        }
    }
}

impl<T> Pair<T> 
where
    T: AsymmetricKeyGenAlgorithm {
    pub fn generate() -> Result<Self> {
        let pair = T::generate()?;
        let pk: Bytes = pair.0
            .as_ref()
            .to_vec()
            .into();
        let pk: public_key::PublicKey<T> = pk.into();
        let sk: Bytes = pair.1
            .as_ref()
            .to_vec()
            .into();
        let sk: secret_key::SecretKey<T> = sk.into();
        let phantom_data: std::marker::PhantomData<T> = std::marker::PhantomData;
        let new: Self = Self {
            phantom_data,
            pk,
            sk
        };
        Ok(new)
    }
}

impl<A> Pair<A> 
where
    A: AsymmetricSignatureAlgorithm {
    pub fn sign<B>(&self, msg: B) -> Result<signed::Signed<A, B>> 
    where
        B: AsRef<[u8]> {
        let msg_bytes: Bytes = msg
            .as_ref()
            .to_vec()
            .into();
        let pk: Bytes = self.pk.as_ref().into();
        let pk: public_key::PublicKey<A> = pk.into();
        let sk: Bytes = self.sk
            .as_ref()
            .to_vec()
            .into();
        let sig: Signature = A::sign(&sk, &msg_bytes)?;
        let sig: signature::Signature<_> = sig.into();
        let signed: signed::Signed<_, _> = signed::Signed::builder()
            .signer(pk)
            .signature(sig)
            .payload(msg)
            .build();
        Ok(signed)
    }
}