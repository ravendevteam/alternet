#![no_std]

use soroban_sdk as so;

#[so::contracttype]
pub enum MemoryStoreKey {
	Owner,
	Unit,
	UnlockTimestamp
}

#[so::contract]
pub struct Main;

#[so::contractimpl]
impl Main {
	pub fn wake(
		environment: so::Env, 
		owner: so::Address, 
		unit: so::Address,
		unlock_timestamp: u64
	) {
		environment.storage().persistent().set(&MemoryStoreKey::Owner, &owner);
		environment.storage().persistent().set(&MemoryStoreKey::Unit, &unit);
		environment.storage().persistent().set(&MemoryStoreKey::UnlockTimestamp, &unlock_timestamp);
	}
	
	pub fn owner(environment: so::Env) -> so::Address {
		environment.storage().persistent().get::<_, so::Address>(&MemoryStoreKey::Owner).expect("awake")
	}
	
	pub fn balance(environment: so::Env) -> u64 {
		let unit = environment.storage().persistent().get::<_, so::Address>(&MemoryStoreKey::Unit).expect("awake");
		// check balance of this contract on unit
		0
	}
	
	pub fn claim() {
		// proof you have delivered packat on behalf of this entity or delivered for this this domain owner
		// check proof
		// unlock amount
		// transfer amount to the claimee from the pool owner
	}
}