use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
pub struct Encrypted<T> {
    phantom_data: std::marker::PhantomData<T>,
    #[deref]
    #[deref_mut]
    content: message::Message
}