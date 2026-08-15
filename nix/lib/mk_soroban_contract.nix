pkgs: config: pname: pkgs.stdenv.mkDerivation rec {
	RUSTFLAGS = "-A warnings";

	pname = pname;
	version = "0.1.0";
	src = ./.;

	cargoDeps = pkgs.rustPlatform.importCargoLock {
		lockFile = ./Cargo.lock;
	};

	nativeBuildInputs = [
		pkgs.rustPlatform.cargoSetupHook
		pkgs.binaryen
		pkgs.rustc
		pkgs.cargo
		pkgs.lld

		config.packages.stellar
	];

	buildPhase = ''
		export HOME=$(mktemp -d)

		stellar contract build --package ${pname}
	'';

	installPhase = ''
		mkdir -p $out/lib

		cp target/wasm32v1-none/release/${pname}.wasm $out/lib/
	'';
}
