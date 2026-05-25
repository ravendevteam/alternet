use clap::Parser as _;

// erc20, domain registration and management
// event sourcing 

pub struct Address(Vec<u8>);

impl From<u64> for Address {
	fn from(value: u64) -> Self {
		let value: Vec<u8> = value.to_le_bytes().to_vec();
		Self(value)
	}
}

#[derive(clap::Parser)]
enum Main {
	Symbol
}

impl Main {
	pub fn consume(self) {
		match self {
			Self::Symbol => {
				
			}
		}
	}
}

fn main() {
	Main::parse().consume();
}