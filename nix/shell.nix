{
	perSystem = { pkgs, config, inputs', ... }: {
		devShells.default = pkgs.mkShell {
			RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

			inputsFrom = [
				config.devShells.rust
			];

			nativeBuildInputs = [
				pkgs.nushell
				pkgs.nixd
				pkgs.nixpkgs-fmt
				pkgs.clippy
				pkgs.cargo
				pkgs.rustc
				pkgs.rust-analyzer
				pkgs.pkg-config
				pkgs.wasm-bindgen-cli
				pkgs.lld
				pkgs.protobuf
				pkgs.docker
				pkgs.mdwatch

				config.packages.stellar
			];

			buildInputs = [
				pkgs.openssl
			];

			shellHook = ''
				nu -c '
					$env.PATH = ($env.PATH | prepend ($env.PWD | path join ".local" "bin"))
					$env.PATH = ($env.PATH | prepend ($env.HOME | path join ".cargo" "bin"))

					try {
						rustup target add wasm32-unknown-unknown
					}
				'
			'';
		};

		rust-project.toolchain = inputs'.fenix.packages.stable.toolchain;
	};
}
