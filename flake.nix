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

		perSystem = { config, pkgs, system, ... }:
		let
			craneLib = inputs.crane.mkLib pkgs;
			craneSrc = craneLib.cleanCargoSource (craneLib.path ./.);
		in {
			packages.vm = pkgs.testers.runNixOSTest {
				name = "vm";
				
				# runs vm with access to internet so deps can be resolved...
				# ^sudo nix run .#vm -L --option sandbox false 
				
				nodes.vm = { ... }: {
					system.stateVersion = "24.05";
					
					virtualisation.docker.enable = true;
					
					environment.etc."workspace".source = ./.;
					
					networking.useDHCP = true;
					
					environment.systemPackages = [
						pkgs.cargo
						pkgs.rustc
						pkgs.pkg-config
						pkgs.openssl
						pkgs.protobuf
						pkgs.git
						pkgs.iptables
						pkgs.tcpdump
						
						config.packages.stellar
					];
				};
				
				# python wtf
				testScript = ''
vm.start()
vm.wait_for_unit("docker.service")

vm.succeed("docker load -i ${config.packages.bootstrapStellarCompatibleImage}")
vm.succeed("cp -r /etc/workspace /root/workspace")
vm.succeed("cd /root/workspace && cargo test --package node --test main -- --nocapture")
				'';
			};
			
			packages.bootstrap = pkgs.rustPlatform.buildRustPackage {
				RUSTFLAGS = "-Awarnings";
				pname = "bootstrap";
				version = "0.1.0";
				src = ./.;
				doCheck = false;
				cargoLock.lockFile = ./Cargo.lock;
				cargoBuildFlags = [
					"--package" "node"
					"--bin" "bootstrap"
					"--features=bootstrap"
					"--no-default-features"
				];
				nativeBuildInputs = [
					pkgs.protobuf
					pkgs.pkg-config
				];
				buildInputs = [
					pkgs.openssl
				];
			};
			
			packages.bootstrapStellarCompatibleImage = pkgs.dockerTools.buildImage {
				name = "bootstrap/stellar";
				tag = config.packages.bootstrap.version;
				copyToRoot = pkgs.buildEnv {
					name = "";
					pathsToLink = [
						"/bin"
						"/etc/ssl/certs"
					];
					paths = [
						pkgs.coreutils
						pkgs.cacert
						config.packages.bootstrap
						config.packages.stellar
					];
				};
				config.Entrypoint = [
					"/bin/bootstrap"
				];
				config.Env = [
					"SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
				];
				config.ExposedPorts."4001/udp" = {};
				config.ExposedPorts."8080/tcp" = {};
				config.WorkingDir = "/workspace";
			};
			
			packages.relay = pkgs.rustPlatform.buildRustPackage {
				RUSTFLAGS = "-Awarnings";
				pname = "relay";
				version = "0.1.0";
				src = ./.;
				doCheck = false;
				cargoLock.lockFile = ./Cargo.lock;
				cargoBuildFlags = [
					"--package" "node"
					"--bin" "relay"
					"--features=relay"
					"--no-default-features"
				];
				nativeBuildInputs = [
					pkgs.protobuf
					pkgs.pkg-config
				];
				buildInputs = [
					pkgs.openssl
				];
			};
			
			packages.relayStellarCompatibleImage = pkgs.dockerTools.buildImage {
				name = "relay/stellar";
				tag = config.packages.relay.version;
				copyToRoot = pkgs.buildEnv {
					name = "";
					pathsToLink = [
						"/bin"
						"/etc/ssl/certs"
					];
					paths = [
						pkgs.coreutils
						pkgs.cacert
						config.packages.relay
						config.packages.stellar
					];
				};
				config.Entrypoint = [
					"/bin/bootstrap"
				];
				config.Env = [
					"SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
				];
				config.ExposedPorts."4001/udp" = {};
				config.ExposedPorts."8080/tcp" = {};
				config.WorkingDir = "/workspace";
			};
			
			packages.stellar =
			let
				pname = "stellar-cli";
				version = "26.0.0";
				architecture.x86_64-linux.target = "x86_64-unknown-linux-gnu";
				architecture.x86_64-linux.sha256 = "sha256-Mcg9s0LRGEsx9lkec5lQ60V1a+RtzXu1fk846R6jLoQ=";
				architecture.x86_64-darwin.target = "x86_64-apple-darwin";
				architecture.x86_64-darwin.sha256 = "sha256-5v6qppaasR8T84XwiTfXhVs2OZuNqtPGd3knmt6hbsg=";
				architecture.aarch64-linux.target = "aarch64-unknown-linux-gnu";
				architecture.aarch64-linux.sha256 = "sha256-q/Wu5hii+ocgX871MrV/MhDzSB0S/j0pDFZnexio79Q=";
				architecture.aarch64-darwin.target = "x86_64-apple-darwin";
				architecture.aarch64-darwin.sha256 = "sha256-OO7oOWuxlCfenDbwfsOVtZzE2P6lupUaC51GPszzq6g=";
				compatibleArchitecture = architecture.${system} or (throw "(unsupported_system=${system})");
				src =
				let
					url = "https://github.com/stellar/stellar-cli/releases/download/v${version}/stellar-cli-${version}-${compatibleArchitecture.target}.tar.gz";
				in pkgs.fetchurl {
					inherit url;
					inherit (compatibleArchitecture) sha256;
				};
				nativeBuildInputs = [
					pkgs.nushell
				] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
					pkgs.autoPatchelfHook
				];
				buildInputs = [
					pkgs.stdenv.cc.cc.lib
				] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
					pkgs.dbus
					pkgs.systemd
					pkgs.openssl
				];
				dontUnpack = true;
				dontBuild = true;
				installPhase = ''
					nu -c '
						mkdir ($env.out | path join "bin")
						tar -xzf $env.src
						cp stellar ($env.out | path join "bin" "stellar")
						chmod +x ($env.out | path join "bin" "stellar")
					'
				'';
			in pkgs.stdenv.mkDerivation {
				inherit pname;
				inherit version;
				inherit src;
				inherit nativeBuildInputs;
				inherit buildInputs;
				inherit dontUnpack;
				inherit dontBuild;
				inherit installPhase;
			};

			devShells.default = pkgs.mkShell {
				RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

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
					pkgs.arion
					pkgs.docker

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

						print "checking for stellar/quickstart `docker` image:"
						let image_check = (do -i { docker images -q stellar/quickstart } | complete)

						if ($image_check.stdout | is-empty) {
							print "stellar/quickstart not found locally: pull in progress:"

							# to fix: needs a version to be fixed not latest to avoid breaking changes or sec vulnerabilities
							docker pull stellar/quickstart:latest
						} else {
							print "stellar/quickstart image aquired"
						}
					'
				'';
			};
		};
	};
}
