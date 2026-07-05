#![no_std]

use soroban_sdk::FromVal as _;
use soroban_sdk::xdr::ToXdr as _;

trait FixedPoint {
	fn fmul(self, rhs: Self, decimals: u32) -> Self;
	fn fdiv(self, rhs: Self, decimals: u32) -> Self;
	fn cast(self, old_decimals: u32, new_decimals: u32) -> Self;
}

impl FixedPoint for soroban_sdk::U256 {
	fn fmul(self, rhs: Self, decimals: u32) -> Self {
		let environment: soroban_sdk::Env = self.env().clone();
		let scaler: u128 = 10;
		let scaler: u128 = scaler.pow(decimals);
		let x: u128 = self.to_u128().expect("should not overflow");
		let y: u128 = rhs.to_u128().expect("should not overflow");
		let out: u128 = x * y;
		let out: u128 = out / scaler;
		let out: Self = Self::from_u128(&environment, out);
		out
	}
	
	fn fdiv(self, rhs: Self, decimals: u32) -> Self {
		let environment: soroban_sdk::Env = self.env().clone();
		let scaler: u128 = 10;
		let scaler: u128 = scaler.pow(decimals);
		let x: u128 = self.to_u128().expect("should not overflow");
		let y: u128 = rhs.to_u128().expect("should not overflow");
		let out: u128 = x * scaler;
		let out: u128 = out / y;
		let out: Self = Self::from_u128(&environment, out);
		out
	}
	
	fn cast(self, old_decimals: u32, new_decimals: u32) -> Self {
		let environment: soroban_sdk::Env = self.env().clone();
		let out: u128 = self.to_u128().expect("should not overflow");
		let out: u128 = if new_decimals > old_decimals {
			let df: u32 = new_decimals - old_decimals;
			let scaler: u128 = 10;
			let scaler: u128 = scaler.pow(df);
			let out: u128 = out * scaler;
			out
		} else if old_decimals > new_decimals {
			let df: u32 = old_decimals - new_decimals;
			let scaler: u128 = 10;
			let scaler: u128 = scaler.pow(df);
			let out: u128 = out / scaler;
			out
		} else {
			out
		};
		let out: Self = Self::from_u128(&environment, out);
		out
	}
}

trait Storage {
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

trait Domain {
	fn entropy(&self, decimals: u32) -> soroban_sdk::U256;
}

impl Domain for soroban_sdk::String {
	fn entropy(&self, decimals: u32) -> soroban_sdk::U256 {
		let environment: soroban_sdk::Env = self.env().clone();
		
		soroban_sdk::U256::from_u32(&environment, 1)
	}
}

trait Core {
	fn configure(
		self,
		tkn: soroban_sdk::Address,
		nft: soroban_sdk::Address,
		mint_min_fee: soroban_sdk::U256,
		mint_max_fee: soroban_sdk::U256,
		renew_min_fee: soroban_sdk::U256,
		renew_max_fee: soroban_sdk::U256,
		harberger_tax_rate: soroban_sdk::U256,
		target_traffic: soroban_sdk::U256
	);
	fn attestation(self, account: ForeignPublicKey) -> Option<soroban_sdk::Address>;
	fn attest(self, local_signer: PublicKey, foreign_signer: ForeignPublicKey, foreign_signature: ForeignSignature);
	fn mint(self, account: soroban_sdk::Address, domain: soroban_sdk::String);
	fn renew(self, domain: soroban_sdk::String);
	fn verify_validity(self, pool_key: u32, proof: soroban_sdk::Bytes);
	fn claim(self, pool_key: u32, proofs: soroban_sdk::Vec<soroban_sdk::Bytes>);
	fn commit(self, amount: soroban_sdk::U256, count: u128) -> (u32, soroban_sdk::Vec<soroban_sdk::Bytes>);
}

impl Core for soroban_sdk::Env {
	fn configure(
		self,
		tkn: soroban_sdk::Address,
		nft: soroban_sdk::Address,
		mint_min_fee: soroban_sdk::U256,
		mint_max_fee: soroban_sdk::U256,
		renew_min_fee: soroban_sdk::U256,
		renew_max_fee: soroban_sdk::U256,
		harberger_tax_rate: soroban_sdk::U256,
		target_traffic: soroban_sdk::U256
	) {
		let state: soroban_sdk::storage::Persistent = self.storage().persistent();
		
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
	
	fn attestation(self, account: ForeignPublicKey) -> Option<soroban_sdk::Address> {
		let state: soroban_sdk::storage::Persistent = self.storage().persistent();
		let ForeignPublicKey(account) = account;
	
		state.get(&MemoryStoreKey::AttestationOwner(account))
	}
	
	fn attest(self, local_signer: PublicKey, foreign_signer: ForeignPublicKey, foreign_signature: ForeignSignature) {
		let state: soroban_sdk::storage::Persistent = self.storage().persistent();
		let event: soroban_sdk::events::Events = self.events();
		let crypt: soroban_sdk::crypto::Crypto = self.crypto();
		let PublicKey(local_signer) = local_signer;
		let ForeignPublicKey(foreign_signer) = foreign_signer;
		let ForeignSignature(foreign_signature) = foreign_signature;
		
		local_signer.require_auth();

		let message: soroban_sdk::Bytes = local_signer.clone().to_xdr(&self);

		crypt.ed25519_verify(&foreign_signer, &message, &foreign_signature);
		
		state.set(&MemoryStoreKey::Attestation(local_signer.clone()), &foreign_signer);
		state.set(&MemoryStoreKey::AttestationOwner(foreign_signer.clone()), &local_signer);
		
		state.extend_memory_store_ttl();
		
		event.publish((soroban_sdk::symbol_short!("attest"), local_signer.clone()), foreign_signer.clone());
	}
	
	fn mint(self, account: soroban_sdk::Address, domain: soroban_sdk::String) {
		let state: soroban_sdk::storage::Persistent = self.storage().persistent();
		let event: soroban_sdk::events::Events = self.events();
		
		// account.require_auth();

		let tkn_public_key: soroban_sdk::Address = state.get(&MemoryStoreKey::Tkn).expect("set during configuration");
		let nft_public_key: soroban_sdk::Address = state.get(&MemoryStoreKey::Nft).expect("set during configuration");
		let min_fee: soroban_sdk::U256 = state.get(&MemoryStoreKey::MintMinFee).expect("set during configuration");
		let max_fee: soroban_sdk::U256 = state.get(&MemoryStoreKey::MintMaxFee).expect("set during configuration");
		let decimals: u32 = 2;
		let entropy_multiplier: soroban_sdk::U256 = domain.entropy(decimals);
		let scaler: u32 = 10;
		let scaler: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&self, scaler);
		let entropy_inversion: soroban_sdk::U256 = scaler.sub(&entropy_multiplier);
		let fee: soroban_sdk::U256 = max_fee.sub(&min_fee);
		let fee: soroban_sdk::U256 = min_fee.add(&fee).fmul(entropy_inversion, decimals);
		
		let balance: soroban_sdk::U256 = self.invoke_contract(
			&tkn_public_key, 
			&soroban_sdk::Symbol::new(&self, "balance_of"), 
			soroban_sdk::vec![
				&self,
				soroban_sdk::Val::from_val(&self, &account)
			]
		);
		
		if balance < fee {
			panic!("insufficient balance to pay mint")
		}
		
		let owner: Option<_> = self.invoke_contract::<Option<soroban_sdk::Address>>(
			&nft_public_key,
			&soroban_sdk::symbol_short!("owner_of"),
			soroban_sdk::vec![
				&self,
				soroban_sdk::Val::from_val(&self, &domain)
			]
		);
		
		if owner.is_some() {
			panic!("domain already owned by someone else")
		}

		self.invoke_contract::<()>(
		    &tkn_public_key,
		    &soroban_sdk::Symbol::new(&self, "transfer"),
		    soroban_sdk::vec![
		        &self,
		        soroban_sdk::Val::from_val(&self, &account),
		        soroban_sdk::Val::from_val(&self, &self.current_contract_address()),
		        soroban_sdk::Val::from_val(&self, &fee)
		    ]
		);

		self.invoke_contract::<()>(
			&nft_public_key,
			&soroban_sdk::symbol_short!("mint"),
			soroban_sdk::vec![
				&self,
				soroban_sdk::Val::from_val(&self, &account),
				soroban_sdk::Val::from_val(&self, &domain)
			]
		);

		event.publish((soroban_sdk::symbol_short!("mint"), account), ());
	}
	
	fn renew(self, domain: soroban_sdk::String) {
		let state: soroban_sdk::storage::Persistent = self.storage().persistent();
		let event: soroban_sdk::events::Events = self.events();
		
		let tkn_public_key: soroban_sdk::Address = state.get(&MemoryStoreKey::Tkn).expect("set during configuration");
		let nft_public_key: soroban_sdk::Address = state.get(&MemoryStoreKey::Nft).expect("set during configuration");
		
		let owner: Option<_> = self.invoke_contract::<Option<soroban_sdk::Address>>(
			&nft_public_key,
			&soroban_sdk::symbol_short!("owner_of"),
			soroban_sdk::vec![
				&self,
				soroban_sdk::Val::from_val(&self, &domain)
			]
		);
		let owner: soroban_sdk::Address = owner.expect("may only renew an owned domain");
		
		//owner.require_auth();
		
		let min_fee: soroban_sdk::U256 = state.get(&MemoryStoreKey::RenewMinFee).expect("set during configuration");
		let max_fee: soroban_sdk::U256 = state.get(&MemoryStoreKey::RenewMaxFee).expect("set during configuration");
		let decimals: u32 = 2;
		let entropy_multiplier: soroban_sdk::U256 = domain.entropy(decimals);
		let scaler: u32 = 10;
		let scaler: soroban_sdk::U256 = soroban_sdk::U256::from_u32(&self, scaler);
		let entropy_inversion: soroban_sdk::U256 = scaler.sub(&entropy_multiplier);
		let fee: soroban_sdk::U256 = max_fee.sub(&min_fee);
		let fee: soroban_sdk::U256 = min_fee.add(&fee).fmul(entropy_inversion, decimals);

		let balance: soroban_sdk::U256 = self.invoke_contract(
			&tkn_public_key, 
			&soroban_sdk::Symbol::new(&self, "balance_of"), 
			soroban_sdk::vec![
				&self,
				soroban_sdk::Val::from_val(&self, &owner)
			]
		);
		
		if balance < fee {
			panic!("insufficient balance to pay renewal")
		}
		
	    self.invoke_contract::<()>(
	        &tkn_public_key,
	        &soroban_sdk::symbol_short!("burn"),
	        soroban_sdk::vec![
		        &self,
		        soroban_sdk::Val::from_val(&self, &owner),
				soroban_sdk::Val::from_val(&self, &fee)
		    ]
	    );
					
		self.invoke_contract::<()>(
			&nft_public_key,
			&soroban_sdk::symbol_short!("renew"),
			soroban_sdk::vec![
				&self,
				soroban_sdk::Val::from_val(&self, &domain)
			]
		);
		
		event.publish((soroban_sdk::symbol_short!("renew"), owner), domain);
	}
	
	fn verify_validity(self, pool_key: u32, proof: soroban_sdk::Bytes) {
		todo!()
	}
	
	fn claim(self, pool_key: u32, proofs: soroban_sdk::Vec<soroban_sdk::Bytes>) {
		todo!()
	}
	
	fn commit(self, amount: soroban_sdk::U256, count: u128) -> (u32, soroban_sdk::Vec<soroban_sdk::Bytes>) {
		todo!()
	}
}

#[soroban_sdk::contracttype]
struct Session {
	dst: ForeignPublicKey,
	dst_signature: ForeignSignature
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
		environment.configure(tkn, nft, mint_min_fee, mint_max_fee, renew_min_fee, renew_max_fee, harberger_tax_rate, target_traffic);
	}

	// attestations must be cachable and stable, so they will eventually have a ttl before having to reconnect them
	pub fn attestation(environment: soroban_sdk::Env, account: ForeignPublicKey) -> Option<soroban_sdk::Address> {
		environment.attestation(account)
	}

	pub fn attest(
		environment: soroban_sdk::Env,
		local_signer: PublicKey,
		foreign_signer: ForeignPublicKey,
		foreign_signature: ForeignSignature
	) {
		environment.attest(local_signer, foreign_signer, foreign_signature);
	}

	pub fn mint(environment: soroban_sdk::Env, account: soroban_sdk::Address, domain: soroban_sdk::String) {
		environment.mint(account, domain);
	}

	pub fn renew(environment: soroban_sdk::Env, domain: soroban_sdk::String) {
		environment.renew(domain);
	}
}
