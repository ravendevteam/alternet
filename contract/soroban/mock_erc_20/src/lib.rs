#![no_std]

#[soroban_sdk::contracttype]
pub enum MemoryStoreKey {
	Admin,
	Name,
	Symbol,
	Decimals,
	TotalSupply,
	Balance(soroban_sdk::Address),
	Allowance(
		soroban_sdk::Address, 
		soroban_sdk::Address
	)
}

#[soroban_sdk::contract]
pub struct Main;

#[soroban_sdk::contractimpl]
impl Main {
	pub fn wake(environment: soroban_sdk::Env, admin: soroban_sdk::Address, name: soroban_sdk::String, symbol: soroban_sdk::String, decimals: soroban_sdk::U256, initial_mint: soroban_sdk::U256) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		if state.has(&MemoryStoreKey::Admin)
		|| state.has(&MemoryStoreKey::Name)
		|| state.has(&MemoryStoreKey::Symbol)
		|| state.has(&MemoryStoreKey::Decimals) {
			panic!("already awoken")
		}

		state.set(&MemoryStoreKey::Admin, &admin);
		state.set(&MemoryStoreKey::Name, &name);
		state.set(&MemoryStoreKey::Symbol, &symbol);
		state.set(&MemoryStoreKey::Decimals, &decimals);
		
		let zero: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);

		state.set(&MemoryStoreKey::TotalSupply, &zero);
		
		if initial_mint > zero {
			state.set(&MemoryStoreKey::TotalSupply, &initial_mint);
			state.set(&MemoryStoreKey::Balance(admin), &initial_mint);
		}
	}

	pub fn owner(environment: soroban_sdk::Env) -> soroban_sdk::Address {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		state.get(&MemoryStoreKey::Admin).expect("set on awakening")
	}
	
	pub fn name(environment: soroban_sdk::Env) -> soroban_sdk::String {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		state.get(&MemoryStoreKey::Name).expect("set on awakening")
	}
	
	pub fn symbol(environment: soroban_sdk::Env) -> soroban_sdk::String {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		state.get(&MemoryStoreKey::Symbol).expect("set on awakening")
	}
	
	pub fn decimals(environment: soroban_sdk::Env) -> soroban_sdk::U256 {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		state.get(&MemoryStoreKey::Decimals).expect("set on awakening")
	}
	
	pub fn total_supply(environment: soroban_sdk::Env) -> soroban_sdk::U256 {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		state.get(&MemoryStoreKey::TotalSupply).expect("set on awakening")
	}
	
	pub fn balance_of(environment: soroban_sdk::Env, account: soroban_sdk::Address) -> soroban_sdk::U256 {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let zero: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);
		
		state.get(&MemoryStoreKey::Balance(account)).unwrap_or(zero)
	}
	
	pub fn allowance(environment: soroban_sdk::Env, account: soroban_sdk::Address, spender: soroban_sdk::Address) -> soroban_sdk::U256 {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let zero: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);
		
		state.get(&MemoryStoreKey::Allowance(account, spender)).unwrap_or(zero)
	}

	pub fn mint(environment: soroban_sdk::Env, account: soroban_sdk::Address, amount: soroban_sdk::U256) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let owner: soroban_sdk::Address = state.get(&MemoryStoreKey::Admin).expect("set on awakening");
		
		owner.require_auth();
		
		let total_supply: soroban_sdk::U256 = Self::total_supply(Clone::clone(&environment));
		let total_supply: soroban_sdk::U256 = total_supply.add(&amount);
		
		state.set(&MemoryStoreKey::TotalSupply, &total_supply);
		
		let balance: soroban_sdk::U256 = Self::balance_of(Clone::clone(&environment), Clone::clone(&account));
		let balance: soroban_sdk::U256 = balance.add(&amount);
		
		state.set(&MemoryStoreKey::Balance(account), &balance);
	}
	
	pub fn burn(environment: soroban_sdk::Env, account: soroban_sdk::Address, amount: soroban_sdk::U256) {		
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let owner: soroban_sdk::Address = state.get(&MemoryStoreKey::Admin).expect("set on awakening");
		
		owner.require_auth();
		
		let balance: soroban_sdk::U256 = Self::balance_of(Clone::clone(&environment), Clone::clone(&account));
		
		if balance < amount {
			panic!("insufficient balance")
		}
		
		let balance: soroban_sdk::U256 = balance.sub(&amount);
		
		state.set(&MemoryStoreKey::Balance(account), &balance);
		
		let total_supply: soroban_sdk::U256 = Self::total_supply(Clone::clone(&environment));
		let total_supply: soroban_sdk::U256 = total_supply.sub(&amount);
		
		state.set(&MemoryStoreKey::TotalSupply, &total_supply);
	}
	
	pub fn approve(environment: soroban_sdk::Env, source: soroban_sdk::Address, spender: soroban_sdk::Address, amount: soroban_sdk::U256) {
		source.require_auth();

		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		
		state.set(&MemoryStoreKey::Allowance(source.clone(), spender.clone()), &amount);

		event.publish((soroban_sdk::symbol_short!("approve"), source, spender), amount);
	}

	pub fn transfer(environment: soroban_sdk::Env, sender: soroban_sdk::Address, recipient: soroban_sdk::Address, amount: soroban_sdk::U256) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		
		sender.require_auth();
		
		let sender_balance: soroban_sdk::U256 = Self::balance_of(Clone::clone(&environment), Clone::clone(&sender));
		
		if sender_balance < amount {
			panic!("insufficient balance")
		}
		
		let sender_balance = sender_balance.sub(&amount);
		let recipient_balance: soroban_sdk::U256 = Self::balance_of(Clone::clone(&environment), Clone::clone(&recipient));
		let recipient_balance: soroban_sdk::U256 = recipient_balance.add(&amount);
	
		state.set(&MemoryStoreKey::Balance(sender.clone()), &sender_balance);
		state.set(&MemoryStoreKey::Balance(recipient.clone()), &recipient_balance);
		
		event.publish((soroban_sdk::symbol_short!("transfer"), sender, recipient), amount);
	}
	
	pub fn transfer_from(environment: soroban_sdk::Env, sender: soroban_sdk::Address, recipient: soroban_sdk::Address, spender: soroban_sdk::Address, amount: soroban_sdk::U256) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		
		spender.require_auth();
		
		let allowance: soroban_sdk::U256 = Self::allowance(environment.clone(), sender.clone(), spender.clone());
		
		if allowance < amount {
			panic!("insufficient allowance")
		}
		
		let sender_balance: soroban_sdk::U256 = Self::balance_of(Clone::clone(&environment), Clone::clone(&sender));
		
		if sender_balance < amount {
			panic!("insufficient balance")
		}
		
		let allowance: soroban_sdk::U256 = allowance.sub(&amount);
		let sender_balance: soroban_sdk::U256 = sender_balance.sub(&amount);	
		let recipient_balance: soroban_sdk::U256 = Self::balance_of(environment.clone(), recipient.clone());
		let recipient_balance: soroban_sdk::U256 = recipient_balance.add(&amount);
		
		state.set(&MemoryStoreKey::Allowance(sender.clone(), spender.clone()), &allowance);
		state.set(&MemoryStoreKey::Balance(sender.clone()), &sender_balance);
		state.set(&MemoryStoreKey::Balance(recipient.clone()), &recipient_balance);
	
		event.publish((soroban_sdk::symbol_short!("transfer"), sender, recipient), amount);
	}
}