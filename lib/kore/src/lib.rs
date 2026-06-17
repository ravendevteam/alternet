pub trait Unpack<T> {
	fn unpack(self) -> T;
}

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
#[derive(Clone)]
pub enum Or<A, B> {
	Lhs(A),
	Rhs(B)
}

impl<A, B> Or<A, B> {
	pub fn from_lhs<C>(x: C) -> Self
	where
		C: Into<A> {
		let x: A = x.into();

		Self::Lhs(x)
	}

	pub fn from_rhs<C>(x: C) -> Self
	where
		C: Into<B> {
		let x: B = x.into();

		Self::Rhs(x)
	}
}