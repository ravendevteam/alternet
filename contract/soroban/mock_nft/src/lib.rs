#![no_std]

#[soroban_sdk::contracttype]
pub enum MemoryStoreKey {
	Owner,
	Ownership(soroban_sdk::String),
	ExpiryTimestamp(soroban_sdk::String),
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
		if environment.storage().persistent().has(&MemoryStoreKey::Owner)
		|| environment.storage().persistent().has(&MemoryStoreKey::Name)
		|| environment.storage().persistent().has(&MemoryStoreKey::Symbol) {
			panic!("already awoken")
		}
		
		environment.storage().persistent().set(&MemoryStoreKey::Owner, &owner);
		environment.storage().persistent().set(&MemoryStoreKey::Name, &name);
		environment.storage().persistent().set(&MemoryStoreKey::Symbol, &symbol);
		
		let n_0: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);

		environment.storage().persistent().set(&MemoryStoreKey::TotalSupply, &n_0);
		environment.events().publish((soroban_sdk::symbol_short!("wake"), owner), (name, symbol));
	}

	pub fn name(environment: soroban_sdk::Env) -> soroban_sdk::String {
		environment.storage().persistent().get(&MemoryStoreKey::Name).expect("set on awakening")
	}

	pub fn symbol(environment: soroban_sdk::Env) -> soroban_sdk::String {
		environment.storage().persistent().get(&MemoryStoreKey::Symbol).expect("set on awakening")
	}
	
	pub fn total_supply(environment: soroban_sdk::Env) -> soroban_sdk::U256 {
		let n_0: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);
		
		environment.storage().persistent().get(&MemoryStoreKey::TotalSupply).unwrap_or(n_0)
	}
	
	// time when the domain becomes mintable and ownership is revoked
	pub fn expiry_timestamp() {
		
	}

	pub fn owner_of(environment: soroban_sdk::Env, domain: soroban_sdk::String) -> soroban_sdk::Address {
		environment.storage().persistent().get(&MemoryStoreKey::Ownership(domain)).expect("awake")
	}

	pub fn mint(environment: soroban_sdk::Env, owner: soroban_sdk::Address, domain: soroban_sdk::String) {
		owner.require_auth();
		
		let key: MemoryStoreKey = MemoryStoreKey::Ownership(Clone::clone(&domain));
		
		if environment.storage().persistent().has(&key) {
			panic!("domain already minted")
		}
		
		let n_1: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 1);
		let total_supply: soroban_sdk::U256 = Self::total_supply(Clone::clone(&environment));
		let total_supply: soroban_sdk::U256 = total_supply.add(&n_1);
		
		environment.storage().persistent().set(&key, &owner);
		environment.storage().persistent().set(&MemoryStoreKey::TotalSupply, &total_supply);
		environment.events().publish((soroban_sdk::symbol_short!("mint"), owner), domain);
	}
	
	pub fn burn(environment: soroban_sdk::Env, owner: soroban_sdk::Address, domain: soroban_sdk::String) {
		owner.require_auth();
		
		let key: MemoryStoreKey = MemoryStoreKey::Ownership(Clone::clone(&domain));
		let key_owner: soroban_sdk::Address = Self::owner_of(Clone::clone(&environment), Clone::clone(&domain));
		
		if key_owner != owner {
			panic!("not authorized")
		}
		
		let n_1: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 1);
		let total_supply: soroban_sdk::U256 = Self::total_supply(Clone::clone(&environment));
		let total_supply: soroban_sdk::U256 = total_supply.sub(&n_1);
		
		environment.storage().persistent().remove(&key);
		environment.storage().persistent().set(&MemoryStoreKey::TotalSupply, &total_supply);
		environment.events().publish((soroban_sdk::symbol_short!("burn"), owner), domain);
	}
	
	pub fn transfer(
		environment: soroban_sdk::Env,
		sender: soroban_sdk::Address,
		recipient: soroban_sdk::Address,
		domain: soroban_sdk::String
	) {
		sender.require_auth();
		
		let key: MemoryStoreKey = MemoryStoreKey::Ownership(Clone::clone(&domain));
		let key_owner: soroban_sdk::Address = Self::owner_of(Clone::clone(&environment), Clone::clone(&domain));
		
		if key_owner != sender {
			panic!("not authorized")
		}
		
		environment.storage().persistent().set(&key, &recipient);
		environment.events().publish((soroban_sdk::symbol_short!("transfer"), sender, recipient), domain);
	}
}
