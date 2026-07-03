#![no_std]

use soroban_sdk::FromVal as _;
use soroban_sdk::xdr::ToXdr as _;

pub trait FixedPoint {
	fn fmul(self, rhs: Self, decimals: soroban_sdk::U256) -> Self;
	fn fdiv(self, rhs: Self, decimals: soroban_sdk::U256) -> Self;
}

impl FixedPoint for soroban_sdk::U256 {
	fn fmul(self, rhs: Self, decimals: soroban_sdk::U256) -> Self {
		let environment = self.env().clone();
		let decimals: u128 = decimals.to_u128().expect("should not overflow");
		let decimals: u32 = decimals.try_into().expect("should not overflow");
		let scaler: u128 = 10;
		let scaler: u128 = scaler.pow(decimals);
		let x: u128 = self.to_u128().expect("should not overflow");
		let y: u128 = rhs.to_u128().expect("should not overflow");
		let out: u128 = x * y;
		let out: u128 = out / scaler;
		let out: Self = Self::from_u128(&environment, out);
		out
	}
	
	fn fdiv(self, rhs: Self, decimals: soroban_sdk::U256) -> Self {
		let environment = self.env().clone();
		let decimals: u128 = decimals.to_u128().expect("should not overflow");
		let decimals: u32 = decimals.try_into().expect("should not overflow");
		let scaler: u128 = 10;
		let scaler: u128 = scaler.pow(decimals);
		let x: u128 = self.to_u128().expect("should not overflow");
		let y: u128 = rhs.to_u128().expect("should not overflow");
		let out: u128 = x * scaler;
		let out: u128 = out / y;
		let out: Self = Self::from_u128(&environment, out);
		out
	}
}

pub trait Storage {
	fn extend_memory_store_ttl(&self);
}

impl Storage for soroban_sdk::storage::Persistent {
	fn extend_memory_store_ttl(&self) {
		self.extend_ttl(&MemoryStoreKey::Tkn, 31500000, 31500000);
		self.extend_ttl(&MemoryStoreKey::Nft, 31500000, 31500000);
		self.extend_ttl(&MemoryStoreKey::MintMinFee, 31500000, 31500000);
		self.extend_ttl(&MemoryStoreKey::MintMaxFee, 31500000, 31500000);
		self.extend_ttl(&MemoryStoreKey::RenewMinFee, 31500000, 31500000);
		self.extend_ttl(&MemoryStoreKey::RenewMaxFee, 31500000, 31500000);
		self.extend_ttl(&MemoryStoreKey::RenewTargetTraffic, 31500000, 31500000);
		self.extend_ttl(&MemoryStoreKey::HarbergerTaxRate, 31500000, 31500000);
	}
}

#[soroban_sdk::contracttype]
pub struct ForeignPublicKey(pub soroban_sdk::BytesN<32>);

#[soroban_sdk::contracttype]
pub struct ForeignSignature(pub soroban_sdk::BytesN<64>);

#[soroban_sdk::contracttype]
pub struct PublicKey(pub soroban_sdk::Address);

#[soroban_sdk::contracttype]
pub struct Proof {
	src: soroban_sdk::Bytes,
	dst: soroban_sdk::Bytes
}

#[soroban_sdk::contracttype]
pub enum MemoryStoreKey {
	Tkn,
	Nft,
	MintMinFee,
	MintMaxFee,
	RenewMinFee,
	RenewMaxFee,
	RenewTargetTraffic,
	HarbergerTaxRate,
	Attestation(soroban_sdk::Address),
	AttestationOwner(soroban_sdk::BytesN<32>),
	BalanceLock(soroban_sdk::Address)
}

#[soroban_sdk::contract]
pub struct Main;

#[soroban_sdk::contractimpl]
impl Main {
	pub fn configure(
		environment: soroban_sdk::Env,
		tkn: soroban_sdk::Address,
		nft: soroban_sdk::Address,
		mint_min_fee: soroban_sdk::U256,
		mint_max_fee: soroban_sdk::U256,
		renew_min_fee: soroban_sdk::U256,
		renew_max_fee: soroban_sdk::U256,
		harberger_tax_rate: soroban_sdk::U256,
		target_traffic: soroban_sdk::U256
	) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		
		if state.has(&MemoryStoreKey::Tkn)
		|| state.has(&MemoryStoreKey::Nft) {
			panic!("already configured")
		}

		state.set(&MemoryStoreKey::Tkn, &tkn);
		state.set(&MemoryStoreKey::Nft, &nft);
		state.set(&MemoryStoreKey::MintMinFee, &mint_min_fee);
		state.set(&MemoryStoreKey::MintMaxFee, &mint_max_fee);
		state.set(&MemoryStoreKey::RenewMinFee, &renew_min_fee);
		state.set(&MemoryStoreKey::RenewMaxFee, &renew_max_fee);
		state.set(&MemoryStoreKey::RenewTargetTraffic, &target_traffic);
		state.set(&MemoryStoreKey::HarbergerTaxRate, &harberger_tax_rate);
		
		state.extend_memory_store_ttl();
	}

	// attestations must be cachable and stable, so they will eventually have a ttl before having to reconnect them
	pub fn attestation(environment: soroban_sdk::Env, account: ForeignPublicKey) -> Option<soroban_sdk::Address> {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let ForeignPublicKey(account) = account;
	
		state.get(&MemoryStoreKey::AttestationOwner(account))
	}

	pub fn attest(
		environment: soroban_sdk::Env,
		local_signer: PublicKey,
		foreign_signer: ForeignPublicKey,
		foreign_signature: ForeignSignature
	) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		let crypt: soroban_sdk::crypto::Crypto = environment.crypto();
		let PublicKey(local_signer) = local_signer;
		let ForeignPublicKey(foreign_signer) = foreign_signer;
		let ForeignSignature(foreign_signature) = foreign_signature;
		
		local_signer.require_auth();

		let message: soroban_sdk::Bytes = local_signer.clone().to_xdr(&environment);

		crypt.ed25519_verify(&foreign_signer, &message, &foreign_signature);
		
		state.set(&MemoryStoreKey::Attestation(local_signer.clone()), &foreign_signer);
		state.set(&MemoryStoreKey::AttestationOwner(foreign_signer.clone()), &local_signer);
		
		state.extend_memory_store_ttl();
		
		event.publish((soroban_sdk::symbol_short!("attest"), local_signer.clone()), foreign_signer.clone());
	}

	pub fn mint(environment: soroban_sdk::Env, account: soroban_sdk::Address, domain: soroban_sdk::String) {
		let state: soroban_sdk::storage::Persistent = environment.storage().persistent();
		let event: soroban_sdk::events::Events = environment.events();
		
		account.require_auth();

		let tkn_public_key: soroban_sdk::Address = state.get(&MemoryStoreKey::Tkn).expect("set during configuration");
		let nft_public_key: soroban_sdk::Address = state.get(&MemoryStoreKey::Nft).expect("set during configuration");
		
		let min_fee: soroban_sdk::U256 = state.get(&MemoryStoreKey::MintMinFee).expect("set during configuration");
		let max_fee: soroban_sdk::U256 = state.get(&MemoryStoreKey::MintMaxFee).expect("set during configuration");
		
		// fixed point multiplication required
		let entropy_multiplier: soroban_sdk::U256 = Self::shannon_entropy(environment.clone(), domain.clone());
		
		
		let balance: soroban_sdk::U256 = environment.invoke_contract(
			&tkn_public_key, 
			&soroban_sdk::Symbol::new(&environment, "balance_of"), 
			soroban_sdk::vec![
				&environment,
				soroban_sdk::Val::from_val(&environment, &account)
			]
		);
		
		
		

		environment.invoke_contract::<Option<soroban_sdk::Address>>(
			&nft_public_key,
			&soroban_sdk::symbol_short!("owner_of"),
			soroban_sdk::vec![
				&environment,
				soroban_sdk::Val::from_val(&environment, &domain)
			]
		)
		.ok_or(()) // turn option to err, where we expect None, if Some, then panic
		.expect_err("domain already owned by someone else");



		// check thaty the domain is not owned by someone else first
		// then check if its expired and been sent back to the mintable pool

	    environment.invoke_contract::<()>(
	        &tkn_public_key,
	        &soroban_sdk::symbol_short!("burn"),
	        soroban_sdk::vec![
		        &environment,
		        soroban_sdk::Val::from_val(&environment, &account),
				soroban_sdk::Val::from_val(&environment, &10000_00) // hardcoded for until proper algorithms are in place
		    ]
	    );

		environment.invoke_contract::<()>(
			&nft_public_key,
			&soroban_sdk::symbol_short!("mint"),
			soroban_sdk::vec![
				&environment,
				soroban_sdk::Val::from_val(&environment, &domain)
			]
		);

		environment.events().publish((soroban_sdk::symbol_short!("mint"), account), ());
	}

	pub fn renew(environment: soroban_sdk::Env) {
		// ...
	}

	pub fn lock(environment: soroban_sdk::Env, owner: soroban_sdk::Address, amount: soroban_sdk::U256) {
		owner.require_auth();

		let token_address: soroban_sdk::Address = environment.storage().persistent().get(&MemoryStoreKey::Tkn).unwrap();

		environment.invoke_contract::<()>(
			&token_address,
			&soroban_sdk::symbol_short!("transfer"),
			soroban_sdk::vec![
				&environment,
				soroban_sdk::Val::from_val(&environment, &owner),
				soroban_sdk::Val::from_val(&environment, &environment.current_contract_address()),
				soroban_sdk::Val::from_val(&environment, &amount)
			]
		);

		let n_0: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);
		let key: MemoryStoreKey = MemoryStoreKey::BalanceLock(owner.clone());
		let old_amount: soroban_sdk::U256 = environment.storage().persistent().get::<_, soroban_sdk::U256>(&key).unwrap_or(n_0);
		let new_amount: soroban_sdk::U256 = old_amount.add(&amount);

		environment.storage().persistent().set::<_, _>(&key, &new_amount);
		environment.events().publish((soroban_sdk::symbol_short!("lock"), owner), amount);
	}

	pub fn claim(environment: soroban_sdk::Env, proof: Proof) {
		// claims reward from locked pool
		// proof is automatically mapped to the cryptographic commitment

	}

	fn fee_rational(
		environment: soroban_sdk::Env,
		min_fee: soroban_sdk::U256,
		max_fee: soroban_sdk::U256,
		traffic: soroban_sdk::U256,
		target_traffic: soroban_sdk::U256
	) -> soroban_sdk::U256 {
		let n_0: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 0);
		let n_1: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 1);

		if traffic <= n_0 {
			return max_fee
		}

		n_0
	}

	fn harberger_tax(environment: soroban_sdk::Env, last_mint: soroban_sdk::U256, tax_rate: soroban_sdk::U256) -> soroban_sdk::U256 {
		let n_100: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&environment, 100);
		let out: soroban_sdk::U256 = last_mint.div(&n_100);
		let out: soroban_sdk::U256 = out.mul(&tax_rate);
		out
	}

	// algorithm to measure complexity of domains
	fn shannon_entropy(environment: soroban_sdk::Env, domain: soroban_sdk::String) -> soroban_sdk::U256 {
		soroban_sdk::U256::from_u32(&environment, 1)
	}
}
