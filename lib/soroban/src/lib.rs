#![no_std]

pub trait FixedPoint {
	fn fmul(self, rhs: Self, precision: u8) -> Self;
	fn fdiv(self, rhs: Self, precision: u8) -> Self;
}

impl FixedPoint for u128 {
	fn fmul(self, rhs: Self, precision: u8) -> Self {
		let precision: u32 = precision.into();
		let scalar: u128 = 10;
		let scalar: u128 = scalar.pow(precision);
		let x: u128 = self;
		let y: u128 = rhs;
		let out: u128 = x * y;
		let out: u128 = out / scalar;
		out	
	}
	
	fn fdiv(self, rhs: Self, precision: u8) -> Self {
		let precision: u32 = precision.into();
		let scalar: u128 = 10;
		let scalar: u128 = scalar.pow(precision);
		let x: u128 = self;
		let y: u128 = rhs;
		let out: u128 = x * scalar;
		let out: u128 = out / y;
		out
	}
}

#[soroban_sdk::contracttype]
#[derive(Clone)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
#[derive(derive_more::Into)]
pub struct ForeignPublicKey(soroban_sdk::BytesN<32>);	// ED25519

#[soroban_sdk::contracttype]
#[derive(Clone)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
#[derive(derive_more::Into)]
pub struct ForeignSignature(soroban_sdk::BytesN<64>);  // ED25519

#[soroban_sdk::contracttype]
#[derive(Clone)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
#[derive(derive_more::Into)]
pub struct PublicKey(soroban_sdk::Address);

#[soroban_sdk::contracttype]
#[derive(Clone)]
#[derive(Copy)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
#[derive(derive_more::Into)]
pub struct Precision(u32);

#[soroban_sdk::contracttype]
#[derive(Clone)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
#[derive(derive_more::Into)]
pub struct Balance<const T: u8 = 2>(soroban_sdk::U256);

impl<const T: u8> Balance<T> {
	pub fn precision(&self) -> Precision {
		let out: u8 = T;
		let out: u32 = out.into();
		let out: Precision = out.into();
		out
	}
}

impl<const A: u8> Balance<A> {
	pub fn cast<const B: u8>(self) -> Balance<B> {
		let environment: soroban_sdk::Env = self.env().clone();
		let value: soroban_sdk::U256 = self.into();
		let value: u128 = value.to_u128().expect("should not overflow");
		
		match A {
			x if x < B => {
				let scalar: u8 = B - A;
				let scalar: u32 = scalar.into();
				let out: u128 = 10;
				let out: u128 = out.pow(scalar);
				let out: u128 = value * out;
				let out: soroban_sdk::U256 = soroban_sdk::U256::from_u128(&environment, out);
				let out: Balance<_> = out.into();
				
				out
			},
			x if x > B => {
				let scaler: u8 = A - B;
				let scaler: u32 = scaler.into();
				let out: u128 = 10;
				let out: u128 = out.pow(scaler);
				let out: u128 = value / out;
				let out: soroban_sdk::U256 = soroban_sdk::U256::from_u128(&environment, out);
				let out: Balance<_> = out.into();
				
				out
			},
			_ => {
				let out: u128 = value;
				let out: soroban_sdk::U256 = soroban_sdk::U256::from_u128(&environment, out);
				let out: Balance<_> = out.into();
				
				out				
			}
		}
	}
}

impl<const T: u8> core::ops::Mul for Balance<T> {
	type Output = Self;
	
	fn mul(self, rhs: Self) -> Self::Output {
		let environment: soroban_sdk::Env = self.env().clone();
		let x: soroban_sdk::U256 = self.into();
		let x: u128 = x.to_u128().expect("should not overflow");
		let y: soroban_sdk::U256 = rhs.into();
		let y: u128 = y.to_u128().expect("should not overflow");
		let out: u128 = x.fmul(y, T);
		let out: soroban_sdk::U256 = soroban_sdk::U256::from_u128(&environment, out);
		let out: Self = out.into();
		out
	}
}

impl<const T: u8> core::ops::Div for Balance<T> {
	type Output = Self;
	
	fn div(self, rhs: Self) -> Self::Output {
		let environment: soroban_sdk::Env = self.env().clone();
		let x: soroban_sdk::U256 = self.into();
		let x: u128 = x.to_u128().expect("should not overflow");
		let y: soroban_sdk::U256 = rhs.into();
		let y: u128 = y.to_u128().expect("should not overflow");
		let out: u128 = x.fdiv(y, T);
		let out: soroban_sdk::U256 = soroban_sdk::U256::from_u128(&environment, out);
		let out: Self = out.into();
		out	
	}
}