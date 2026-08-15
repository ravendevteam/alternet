{
	inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
	inputs.flake-parts.url = "github:hercules-ci/flake-parts";
	inputs.rust-flake.url = "github:juspay/rust-flake";
	inputs.rust-flake.inputs.nixpkgs.follows = "nixpkgs";
	inputs.fenix.url = "github:nix-community/fenix";
	inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";
	inputs.crane.url = "github:ipetkov/crane";
	inputs.crane.inputs.nixpkgs.follows = "nixpkgs";

	outputs = inputs @ { flake-parts, ... }: flake-parts.lib.mkFlake {
		inherit inputs;
	} {
		systems = inputs.nixpkgs.lib.systems.flakeExposed;

		imports = [
			inputs.rust-flake.flakeModules.default
			inputs.rust-flake.flakeModules.nixpkgs

			./nix/check/end_to_end.nix
			./nix/check/soroban_mock_dns.nix
			./nix/check/soroban_mock_erc_20.nix
			./nix/check/soroban_mock_nft.nix
			./nix/check/transport.nix
			./nix/pkg/node.nix
			./nix/pkg/soroban_mock_dns.nix
			./nix/pkg/soroban_mock_erc_20.nix
			./nix/pkg/soroban_mock_nft.nix
			./nix/pkg/stellar.nix
			./nix/pkg/stellar_testnet_image.nix
			./nix/shell.nix
		];
	};
}
