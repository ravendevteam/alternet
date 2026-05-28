#![no_std]

use soroban_sdk as so;

#[so::contracttype]
pub enum MemoryStoreKey {
	Name,
	Symbol,
	Unit,
	Owner(so::String)
}

#[so::contract]
pub struct Main;

#[so::contractimpl]
impl Main {
	fn wake(
		environment: so::Env, 
		name: so::String, 
		symbol: so::String, 
		unit: so::Address,
		min_fee: u64,
		max_fee: u64,
		traffic_target: u64
	) {
		environment.storage().instance().set(&MemoryStoreKey::Name, &name);
		environment.storage().instance().set(&MemoryStoreKey::Symbol, &symbol);
		environment.storage().instance().set(&MemoryStoreKey::Unit, &unit);
	}

	fn name(environment: so::Env) -> so::String {
		environment.storage().instance().get(&MemoryStoreKey::Name).expect("awakened")
	}

	fn symbol(environment: so::Env) -> so::String {
		environment.storage().instance().get(&MemoryStoreKey::Symbol).expect("awakened")
	}

	// owner_of("google")
	fn owner_of(environment: so::Env, domain: so::String) -> so::Address {
		environment.storage().instance().get(&MemoryStoreKey::Owner(domain)).expect("awake")
	}

	// renew existing domain
	pub fn renew(environment: so::Env, domain: so::String) {
		// get caller
		// check if caller owns the domain
		// if they do
	}

	pub fn mint(environment: so::Env, owner: so::Address, domain: so::String) {
		// check that there is sufficient balance to mint
		// check that is has not been minted before
		// include expirery
	}
}
