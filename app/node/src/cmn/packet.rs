#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
pub struct Packet<T> {
	phantom_data: std::marker::PhantomData<T>,
	pub peer: libp2p::PeerId,
	#[deref]
	#[deref_mut]
	pub content: bytes::Bytes
}