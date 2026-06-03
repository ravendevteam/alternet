

// erc20, domain registration and management
// event sourcing 

pub struct Address(Vec<u8>);

impl From<u64> for Address {
	fn from(value: u64) -> Self {
		let value: Vec<u8> = value.to_le_bytes().to_vec();
		Self(value)
	}
}



#[rocket::get("/token_name")]
fn name() -> &'static str {
	""
}

#[rocket::get("/token/mint")]
fn token_mint(account: u32) {
	
}


struct M {
	owner: u32,
	domain_to_ownership: std::collections::HashMap<String, u32>,
	domain_to_expiry_timestamp: std::collections::HashMap<String, u64>,
	name: String,
	symbol: String,
	total_supply: u64
}

impl M {
	
}

fn c(domain: &str) -> u32 {
	let chars: Vec<_> = domain.chars().collect();
	let len: usize = chars.len();
	if len == 0 {
		return 1
	}
	let mut freq = std::collections::HashMap::new();
	for char in &chars {
		freq.entry(char).or_insert(0);
	}
	
	let mut entropy = 0;
	let total_len = len as u32;
	
	
	0
}

fn main() {
	std::process::Command::new("stellar").args(["container", "start", "local"]);
	std::process::Command::new("stellar").args(["keys", "generate", "steve"]);
	std::process::Command::new("stellar").args(["keys", "fund", "steve", "--network", "testnet"]);
	
	std::process::Command::new("stellar").args([
		"contract", "invoke",
		"--id", "<id>",
		"--source", "steve",
		"--network", "testnet",
		"--", 
		"sign_attestation",
		"--owner", "<pk>",
		"--foreign_public_key", "",
		"--foreign_signature", ""
	]);
}