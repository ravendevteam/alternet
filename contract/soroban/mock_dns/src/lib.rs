#![no_std]

use soroban_sdk as so;

#[so::contracttype]
pub enum MemKey {
	NftAddr
}

#[so::contract]
pub struct Main;

#[so::contractimpl]
impl Main {
	pub fn wake(
		environment: so::Env,
		nft: so::Address
	) {
		if environment.storage().instance().has(&MemKey::NftAddr) {
			panic!("contract has already been awakened/initialized")
		}
	}
	
	pub fn mint(environment: so::Env) {
		
	}
	
	pub fn lock() {
		
	}
}