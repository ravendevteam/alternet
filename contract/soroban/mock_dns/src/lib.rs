#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned as _;
use soroban_sdk::FromVal as _;
use soroban_sdk::xdr::ToXdr as _;

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
	RenewMinFee,
	RenewMaxFee,
	RenewTargetTraffic,
	Attestation(soroban_sdk::Address),
	AttestationOwner(soroban_sdk::BytesN<32>)
}

#[soroban_sdk::contract]
pub struct Main;

#[soroban_sdk::contractimpl]
impl Main {
	pub fn wake(
		environment: soroban_sdk::Env,
		tkn: soroban_sdk::Address,
		nft: soroban_sdk::Address,
		min_fee: soroban_sdk::U256,
		max_fee: soroban_sdk::U256,
		harberger_tax_rate: soroban_sdk::U256,
		target_traffic: soroban_sdk::U256
	) {
		if environment.storage().persistent().has(&MemoryStoreKey::Tkn) 
		|| environment.storage().persistent().has(&MemoryStoreKey::Nft) {
			panic!("awoken")
		}

		environment.storage().persistent().set(&MemoryStoreKey::Tkn, &tkn);
		environment.storage().persistent().set(&MemoryStoreKey::Nft, &nft);
		environment.storage().persistent().set(&MemoryStoreKey::RenewMinFee, &min_fee);
		environment.storage().persistent().set(&MemoryStoreKey::RenewMaxFee, &max_fee);
		environment.storage().persistent().set(&MemoryStoreKey::RenewTargetTraffic, &target_traffic);
	}
	
	pub fn attestation(environment: soroban_sdk::Env, foreign_public_key: soroban_sdk::BytesN<32>) -> Option<soroban_sdk::Address> {		
		environment.storage().persistent().get(&MemoryStoreKey::AttestationOwner(foreign_public_key))
	}
	
	pub fn sign_attestation(
		environment: soroban_sdk::Env, 
		owner: soroban_sdk::Address, 
		foreign_public_key: ForeignPublicKey,
		foreign_signature: ForeignSignature
	) {
		owner.require_auth();
		
		let message: soroban_sdk::Bytes = owner.to_owned().to_xdr(&environment);
		let raw_pub_key: &soroban_sdk::BytesN<32> = &foreign_public_key.0;
		let raw_sig: &soroban_sdk::BytesN<64> = &foreign_signature.0;
		
		environment.crypto().ed25519_verify(raw_pub_key, &message, raw_sig);
		environment.storage().persistent().set(&MemoryStoreKey::Attestation(Clone::clone(&owner)), &foreign_public_key);
		environment.storage().persistent().set(&MemoryStoreKey::AttestationOwner(Clone::clone(&foreign_public_key)), &owner);
		environment.events().publish((soroban_sdk::symbol_short!("attest"), owner), foreign_public_key);
	}
	
	pub fn mint(environment: soroban_sdk::Env, account: soroban_sdk::Address) {
		account.require_auth();
	
		let event: soroban_sdk::events::Events = environment.events();
		
		let token_address: soroban_sdk::Address = environment.storage().persistent().get(&MemoryStoreKey::Tkn).unwrap();

	    environment.invoke_contract::<()>(
	        &token_address, 
	        &soroban_sdk::symbol_short!("burn"),
	        soroban_sdk::vec![
		        &environment, 
		        soroban_sdk::Val::from_val(&environment, &account),
				soroban_sdk::Val::from_val(&environment, &299)
		    ]
	    );
		
		environment.events().publish((soroban_sdk::symbol_short!("mint"), account), ());
	}
	
	pub fn renew(environment: soroban_sdk::Env) {
		Self::fee_rational(min_fee, max_fee, traffic, target_traffic);
		
	    environment.invoke_contract(
	        &token_address, 
	        &soroban_sdk::symbol_short!("burn"),
	        soroban_sdk::vec![
		        &environment, 
		        soroban_sdk::Val::from_val(&environment, &account),
				soroban_sdk::Val::from_val(&environment, &)
		    ]
	    );
	}

	pub fn lock(environment: soroban_sdk::Env, owner: soroban_sdk::Address, amount: soroban_sdk::U256) {
		owner.require_auth();
		
		
	}
	
	pub fn submit_proof(environment: soroban_sdk::Env, proof: Proof) {
		
	}
	
	pub fn claim(environment: soroban_sdk::Env) {
		
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
		
		

		()
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
