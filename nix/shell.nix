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
				config.packages.dev
			];

			buildInputs = [
				pkgs.openssl
			];

			shellHook = ''
				export PATH="$PWD/.local/bin:$HOME/.cargo/bin:$PATH"

				dev welcome
			'';
		};

		rust-project.toolchain = inputs'.fenix.packages.stable.toolchain;
	};
}
