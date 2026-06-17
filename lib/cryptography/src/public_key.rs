use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct PublicKey<T> {
    phantom_data: std::marker::PhantomData<T>,
    #[deref]
    #[deref_mut]
    content: bytes::Bytes
}