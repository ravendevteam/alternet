use super::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct Pair<T> {
    phantom_data: std::marker::PhantomData<T>,
    public_key: public_key::PublicKey<T>,
    secret_key: secret_key::SecretKey<T>
}

impl<T> Pair<T>
where
    T: AsymmetricKeyGenAlgorithm {
    pub fn generate() -> Result<Self> {
        T::generate()
    }
}

impl<T> From<(public_key::PublicKey<T>, secret_key::SecretKey<T>)> for Pair<T> {
	fn from(value: (public_key::PublicKey<T>, secret_key::SecretKey<T>)) -> Self {
		let (public_key, secret_key) = value;
		Self {
			phantom_data: std::marker::PhantomData,
			public_key,
			secret_key
		}
	}
}

impl<T> lib_kore::Unpack<(public_key::PublicKey<T>, secret_key::SecretKey<T>)> for Pair<T> {
	fn unpack(self) -> (public_key::PublicKey<T>, secret_key::SecretKey<T>) {
		let Self {
			public_key,
			secret_key,
			..
		} = self;
		(public_key, secret_key)
	}
}