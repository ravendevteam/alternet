#![no_std]

#[soroban_sdk::contracttype]
pub enum MemoryStoreKey {
	Admin,
	Ownership(soroban_sdk::String),
	OwnershipExpiryTimestamp(soroban_sdk::String),
	Name,
	Symbol,
	TotalSupply
}

#[soroban_sdk::contract]
pub struct Main;

#[soroban_sdk::contractimpl]
impl Main {
	pub fn configure(environment: soroban_sdk::Env, admin: soroban_sdk::Address, name: soroban_sdk::String, symbol: soroban_sdk::String) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();

		if state.has(&MemoryStoreKey::Admin)
		|| state.has(&MemoryStoreKey::Name)
		|| state.has(&MemoryStoreKey::Symbol)
	 	|| state.has(&MemoryStoreKey::TotalSupply) {
			panic!("already configured")
		}

		state.set(&MemoryStoreKey::Admin, &admin);
		state.set(&MemoryStoreKey::Name, &name);
		state.set(&MemoryStoreKey::Symbol, &symbol);

		let n_0: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);

		state.set(&MemoryStoreKey::TotalSupply, &n_0);

		event.publish((soroban_sdk::symbol_short!("wake"), admin), (name, symbol));
	}

	pub fn name(environment: soroban_sdk::Env) -> soroban_sdk::String {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();

		state.get(&MemoryStoreKey::Name).expect("set during configuration")
	}

	pub fn symbol(environment: soroban_sdk::Env) -> soroban_sdk::String {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();

		state.get(&MemoryStoreKey::Symbol).expect("set during configuration")
	}

	pub fn total_supply(environment: soroban_sdk::Env) -> soroban_sdk::U256 {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let n_0: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);

		state.get(&MemoryStoreKey::TotalSupply).unwrap_or(n_0)
	}

	pub fn expiry_timestamp(environment: soroban_sdk::Env, domain: soroban_sdk::String) -> Option<u64> {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();

		state.get(&MemoryStoreKey::OwnershipExpiryTimestamp(domain))
	}

	pub fn owner_of(environment: soroban_sdk::Env, domain: soroban_sdk::String) -> Option<soroban_sdk::Address> {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();

		if let Some(expiration) = Self::expiry_timestamp(environment.clone(), domain.clone()) {
			let now: u64 = environment.ledger().timestamp();

			if now >= expiration {
				return None
			}
		}

		state.get(&MemoryStoreKey::Ownership(domain))
	}

	pub fn mint(environment: soroban_sdk::Env, owner: soroban_sdk::Address, domain: soroban_sdk::String) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		let admin: soroban_sdk::Address = state.get(&MemoryStoreKey::Admin).expect("set during configuration");

		admin.require_auth();

		if Self::owner_of(environment.clone(), domain.clone()).is_some() {
			panic!("domain unavailable")
		}

		let own_key: MemoryStoreKey = MemoryStoreKey::Ownership(domain.clone());
		let exp_key: MemoryStoreKey = MemoryStoreKey::OwnershipExpiryTimestamp(domain.clone());

		let expiration: u64 = environment.ledger().timestamp() + 31500000; // roughly one year from now

		let n_1: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 1);
		let total_supply: soroban_sdk::U256 = Self::total_supply(Clone::clone(&environment));
		let total_supply: soroban_sdk::U256 = total_supply.add(&n_1);

		state.set(&own_key, &owner);
		state.set(&exp_key, &expiration);
		state.set(&MemoryStoreKey::TotalSupply, &total_supply);

		event.publish((soroban_sdk::symbol_short!("mint"), owner), (domain, expiration));
	}

	pub fn burn(environment: soroban_sdk::Env, owner: soroban_sdk::Address, domain: soroban_sdk::String) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		let admin: soroban_sdk::Address = state.get(&MemoryStoreKey::Admin).expect("set during configuration");

		admin.require_auth();

		let own_key: MemoryStoreKey = MemoryStoreKey::Ownership(domain.clone());
		let exp_key: MemoryStoreKey = MemoryStoreKey::OwnershipExpiryTimestamp(domain.clone());

		let key_owner: soroban_sdk::Address = Self::owner_of(environment.clone(), domain.clone()).expect("domain does not exist or has already expired");

		if key_owner != owner {
			panic!("not authorized")
		}

		let n_1: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 1);
		let total_supply: soroban_sdk::U256 = Self::total_supply(environment.clone());
		let total_supply: soroban_sdk::U256 = total_supply.sub(&n_1);

		state.remove(&own_key);
		state.remove(&exp_key);
		state.set(&MemoryStoreKey::TotalSupply, &total_supply);

		event.publish((soroban_sdk::symbol_short!("burn"), owner), domain);
	}

	pub fn transfer(environment: soroban_sdk::Env, sender: soroban_sdk::Address, recipient: soroban_sdk::Address, domain: soroban_sdk::String ) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();

		sender.require_auth();

		let key: soroban_sdk::String = domain.clone();
		let key: MemoryStoreKey = MemoryStoreKey::Ownership(key);
		let key_owner: soroban_sdk::Address = Self::owner_of(environment.clone(), domain.clone()).expect("domain does not exist or has already expired");

		if key_owner != sender {
			panic!("not authorized")
		}

		state.set(&key, &recipient);

		event.publish((soroban_sdk::symbol_short!("transfer"), sender, recipient), domain);
	}
}
