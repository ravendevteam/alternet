{
	inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
	inputs.flake-parts.url = "github:hercules-ci/flake-parts";
	inputs.crane.url = "github:ipetkov/crane";

	outputs = inputs @ { flake-parts, ... }:
	flake-parts.lib.mkFlake {
		inherit inputs;
	} {
		systems = [
			"x86_64-linux"
			"x86_64-darwin"
			"aarch64-linux"
			"aarch64-darwin"
		];

		perSystem = { pkgs, system, ... }:
		let
			crane_lib = inputs.crane.mkLib pkgs;
			crane_src = crane_lib.cleanCargoSource (crane_lib.path ./.);
		in {
			devShells.default = pkgs.mkShell {
				RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

				nativeBuildInputs = [
					pkgs.nixd
					pkgs.nixpkgs-fmt
					pkgs.clippy
					pkgs.cargo
					pkgs.rustc
					pkgs.rust-analyzer
					pkgs.pkg-config
				];

				buildInputs = [
					pkgs.openssl
				];

				shellHook = ''
					export PATH="$HOME/.cargo/bin:$PATH"
				'';
			};
		};
	};
}
