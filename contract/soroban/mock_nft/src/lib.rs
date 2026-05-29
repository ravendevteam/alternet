#![no_std]

#[soroban_sdk::contracttype]
pub enum MemoryStoreKey {
	Owner,
	Ownership(soroban_sdk::String),
	Name,
	Symbol,
	TotalSupply
}

#[soroban_sdk::contract]
pub struct Main;

#[soroban_sdk::contractimpl]
impl Main {
	pub fn wake(
		environment: soroban_sdk::Env,
		owner: soroban_sdk::Address,
		name: soroban_sdk::String, 
		symbol: soroban_sdk::String
	) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		
		if state.has(&MemoryStoreKey::Owner)
		|| state.has(&MemoryStoreKey::Name)
		|| state.has(&MemoryStoreKey::Symbol) {
			panic!("already awoken")
		}
		
		state.set(&MemoryStoreKey::Owner, &owner);
		state.set(&MemoryStoreKey::Name, &name);
		state.set(&MemoryStoreKey::Symbol, &symbol);
		
		let zero: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);

		state.set(&MemoryStoreKey::TotalSupply, &zero);
		event.publish((soroban_sdk::symbol_short!("wake"), owner), (name, symbol));
	}

	pub fn name(environment: soroban_sdk::Env) -> soroban_sdk::String {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		state.get(&MemoryStoreKey::Name).expect("set on awakening")
	}

	pub fn symbol(environment: soroban_sdk::Env) -> soroban_sdk::String {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		state.get(&MemoryStoreKey::Symbol).expect("set on awakening")
	}
	
	pub fn total_supply(environment: soroban_sdk::Env) -> soroban_sdk::U256 {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let zero: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);
		
		state.get(&MemoryStoreKey::TotalSupply).unwrap_or(zero)
	}

	pub fn owner_of(environment: soroban_sdk::Env, domain: soroban_sdk::String) -> soroban_sdk::Address {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		state.get(&MemoryStoreKey::Ownership(domain)).expect("awake")
	}

	pub fn mint(environment: soroban_sdk::Env, owner: soroban_sdk::Address, domain: soroban_sdk::String) {
		owner.require_auth();
		
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		let key: MemoryStoreKey = MemoryStoreKey::Ownership(Clone::clone(&domain));
		
		if state.has(&key) {
			panic!("domain already minted")
		}
		
		let one: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 1);
		let total_supply: soroban_sdk::U256 = Self::total_supply(Clone::clone(&environment));
		let total_supply: soroban_sdk::U256 = total_supply.add(&one);
		
		state.set(&key, &owner);
		state.set(&MemoryStoreKey::TotalSupply, &total_supply);
		event.publish((soroban_sdk::symbol_short!("mint"), owner), domain);
	}
	
	pub fn burn(environment: soroban_sdk::Env, owner: soroban_sdk::Address, domain: soroban_sdk::String) {
		owner.require_auth();
		
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		let key: MemoryStoreKey = MemoryStoreKey::Ownership(Clone::clone(&domain));
		let key_owner: soroban_sdk::Address = Self::owner_of(Clone::clone(&environment), Clone::clone(&domain));
		
		if key_owner != owner {
			panic!("not authorized")
		}
		
		let one: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 1);
		let total_supply: soroban_sdk::U256 = Self::total_supply(Clone::clone(&environment));
		let total_supply: soroban_sdk::U256 = total_supply.sub(&one);
		
		state.remove(&key);
		state.set(&MemoryStoreKey::TotalSupply, &total_supply);
		event.publish((soroban_sdk::symbol_short!("burn"), owner), domain);
	}
	
	pub fn transfer(
		environment: soroban_sdk::Env,
		sender: soroban_sdk::Address,
		recipient: soroban_sdk::Address,
		domain: soroban_sdk::String
	) {
		sender.require_auth();
		
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		let key: MemoryStoreKey = MemoryStoreKey::Ownership(Clone::clone(&domain));
		let key_owner: soroban_sdk::Address = Self::owner_of(Clone::clone(&environment), Clone::clone(&domain));
		
		if key_owner != sender {
			panic!("not authorized")
		}
		
		state.set(&key, &recipient);
		event.publish((soroban_sdk::symbol_short!("transfer"), sender, recipient), domain);
	}
}
