#![no_std]

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
	AttestationOwner(soroban_sdk::BytesN<32>),
	BalanceLock(soroban_sdk::Address)
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

	// attestations must be cachable and stable, they must be immutable
	pub fn attestation(environment: soroban_sdk::Env, account: ForeignPublicKey) -> Option<soroban_sdk::Address> {
		environment.storage().persistent().get(&MemoryStoreKey::AttestationOwner(account.0))
	}

	pub fn attest(
		environment: soroban_sdk::Env,
		local_signer: soroban_sdk::Address,
		foreign_signer: ForeignPublicKey,
		foreign_signature: ForeignSignature
	) {
		local_signer.require_auth();

		let message: soroban_sdk::Bytes = local_signer.clone().to_xdr(&environment);

		let foreign_signer: &soroban_sdk::BytesN<32> = &foreign_signer.0;
		let foreign_signature: &soroban_sdk::BytesN<64> = &foreign_signature.0;

		environment.crypto().ed25519_verify(foreign_signer, &message, foreign_signature);
		environment.storage().persistent().set(&MemoryStoreKey::Attestation(local_signer.clone()), &foreign_signer);
		environment.storage().persistent().set(&MemoryStoreKey::AttestationOwner(foreign_signer.clone()), &local_signer);
		environment.events().publish((soroban_sdk::symbol_short!("attest"), local_signer.clone()), foreign_signer.clone());
	}

	pub fn mint(environment: soroban_sdk::Env, account: soroban_sdk::Address, domain: soroban_sdk::String) {
		account.require_auth();

		let token_address: soroban_sdk::Address = environment.storage().persistent().get(&MemoryStoreKey::Tkn).unwrap();
		let domain_nft_address = environment.storage().persistent().get(&MemoryStoreKey::Nft).unwrap();

		environment.invoke_contract::<Option<soroban_sdk::Address>>(
			&domain_nft_address,
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
	        &token_address,
	        &soroban_sdk::symbol_short!("burn"),
	        soroban_sdk::vec![
		        &environment,
		        soroban_sdk::Val::from_val(&environment, &account),
				soroban_sdk::Val::from_val(&environment, &10000_00) // hardcoded for until proper algorithms are in place
		    ]
	    );

		environment.invoke_contract::<()>(
			&domain_nft_address,
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
