use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Signed<A, B> {
    #[serde(skip)]
    phantom_data: std::marker::PhantomData<(A, B)>,
    pub signer: public_key::PublicKey<A>,
    pub signature: signature::Signature<A>,
    pub payload: B
}

#[bon::bon]
impl<A, B> Signed<A, B> {
    #[builder]
    pub fn new(signer: public_key::PublicKey<A>, signature: signature::Signature<A>, payload: B) -> Self {
        let phantom_data: std::marker::PhantomData<_> = std::marker::PhantomData;
        Self {
            phantom_data,
            signer,
            signature,
            payload
        }
    }
}

impl<A, B> Signed<A, B> 
where
    A: AsymmetricSignatureAlgorithm,
    B: AsRef<[u8]> {
    pub fn verify(&self) -> Result<bool> {
        A::verify(
            &self.signer.as_ref().into(),
            &self.payload.as_ref().into(),
            &self.signature.as_ref().into()
        )
    }
}