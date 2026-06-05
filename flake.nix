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
			
			mkNode = role: pkgs.rustPlatform.buildRustPackage {
				pname = role;
				version = "0.1.0";
				src = ./.;
				doCheck = false;
				cargoLock.lockFile = ./Cargo.lock;
				cargoBuildFlags = [
					"--package" "node"
					"--bin" role
					"--features=${role}"
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
		in {
			packages.e2e = 
			let
				wan = 1;
				isp_wan_ip = "192.168.1.254";
				chain_wan_ip = "192.168.1.3";
				bootstrap_ip = "192.168.1.1";
				relay_ip = "192.168.1.2";
				client_lan = 2;
				client_ip = "192.168.2.1";
				client_router_wan_eth = "eth1";
				client_router_lan_eth = "eth2";
				client_router_wan_ip = "192.168.1.252";
				client_router_lan_ip = "192.168.2.254";
				server_lan = 3;
				server_router_wan_ip = "192.168.1.253";
			in pkgs.testers.runNixOSTest {
				name = "e2e";			
				
				nodes.isp.system.stateVersion = "26.05";
				
				nodes.isp.virtualisation.vlans = [
					wan
				];
				
				nodes.isp.boot.kernel.sysctl."net.ipv4.ip_forward" = 1;
				
				nodes.isp.networking.useDHCP = false;
				nodes.isp.networking.interfaces.eth1.ipv4.addresses = [{ address = isp_wan_ip; prefixLength = 24; }];
				nodes.isp.networking.interfaces.eth1.ipv4.routes = [
					{ address = "192.168.2.0"; prefixLength = 24; via = client_router_wan_ip; }
					{ address = "192.168.3.0"; prefixLength = 24; via = server_router_wan_ip; }
				];
				
				nodes.chain.system.stateVersion = "26.05";
				
				nodes.chain.virtualisation.diskSize = 8192;
				nodes.chain.virtualisation.memorySize = 2048;
				nodes.chain.virtualisation.docker.enable = true;
				nodes.chain.virtualisation.docker.autoPrune.enable = true;
				nodes.chain.virtualisation.vlans = [
					wan
				];
				
				nodes.chain.networking.useDHCP = false;
				nodes.chain.networking.defaultGateway = isp_wan_ip;
				nodes.chain.networking.interfaces.eth1.ipv4.addresses = [{ address = chain_wan_ip; prefixLength = 24; }];
				nodes.chain.networking.firewall.allowedTCPPorts = [
					8080
				];
				
				nodes.chain.environment.systemPackages = [
					pkgs.nushell
					pkgs.docker
					
					config.packages.stellar
				];

				nodes.bootstrap.system.stateVersion = "26.05";
				
				nodes.bootstrap.virtualisation.vlans = [
					wan
				];
				
				nodes.bootstrap.networking.useDHCP = false;
				nodes.bootstrap.networking.defaultGateway = isp_wan_ip;
				nodes.bootstrap.networking.interfaces.eth1.ipv4.addresses = [{ address = bootstrap_ip; prefixLength = 24; }];
				
				nodes.bootstrap.environment.systemPackages = [
					pkgs.nushell
					
					config.packages.bootstrap
					config.packages.stellar
				];
				
				nodes.relay.system.stateVersion = "26.05";
				
				nodes.relay.virtualisation.vlans = [
					wan
				];
				
				nodes.relay.environment.systemPackages = [
					pkgs.nushell
					
					config.packages.relay
					config.packages.stellar
				];
				
				nodes.relay.networking.useDHCP = false;
				nodes.relay.networking.defaultGateway = isp_wan_ip;
				nodes.relay.networking.interfaces.eth1.ipv4.addresses = [{ address = relay_ip; prefixLength = 24; }];
				
				nodes.client.system.stateVersion = "26.05";
				
				nodes.client.virtualisation.vlans = [
					client_lan
				];
				
				nodes.client.networking.useDHCP = false;
				nodes.client.networking.defaultGateway = client_router_lan_ip;
				nodes.client.networking.interfaces.eth1.ipv4.addresses = [{ address = client_ip; prefixLength = 24; }];
				
				nodes.client_router.system.stateVersion = "26.05";
				
				nodes.client_router.virtualisation.vlans = [
					wan
					client_lan
				];
				
				nodes.client_router.boot.kernel.sysctl."net.ipv4.ip_forward" = 1;
				
				nodes.client_router.networking.useDHCP = false;
				nodes.client_router.networking.firewall.enable = true;
				nodes.client_router.networking.firewall.allowPing = true;
				nodes.client_router.networking.firewall.extraCommands = ''
					iptables -t nat -A POSTROUTING -o ${client_router_wan_eth} -p udp -j MASQUERADE --random-fully
					iptables -A FORWARD -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
					iptables -A FORWARD -i ${client_router_lan_eth} -o ${client_router_wan_eth} -j ACCEPT
					iptables -A FORWARD -d ${bootstrap_ip} -p udp --dport 4001 -j ACCEPT
					iptables -A FORWARD -d ${relay_ip} -p udp --dport 4001 -j ACCEPT
					iptables -A FORWARD -d ${server_router_wan_ip} -p udp --dport 4001 -j DROP
					iptables -P FORWARD DROP
					
					tc qdisc add dev ${client_router_wan_eth} root netem delay 100ms 10ms loss 1%
				'';
				
				nodes.client_router.networking.interfaces.eth1.ipv4.addresses = [{ address = client_router_wan_ip; prefixLength = 24; }];
				nodes.client_router.networking.interfaces.eth2.ipv4.addresses = [{ address = client_router_lan_ip; prefixLength = 24; }];
				
				# --- call grpc to interact with the nodes within the vm
				# grpcurl localhost:8080 list
				# grpcurl localhost:8080 list your.package.ServiceName
				# grpcurl -d '{"field_name": "value"}' localhost:8080 your.package.ServiceName/MethodName
				
				testScript = pkgs.lib.concatLines [
					"isp.start()"
					"isp.wait_for_unit(\"network.target\")"
					
					"chain.start()"
					"chain.wait_for_unit(\"network.target\")"
					"chain.wait_for_unit(\"docker.service\")"
					"chain.succeed(\"nu -c 'docker load --input ${config.packages.stellar_testnet_image}'\")"
					"chain.succeed(\"nu -c 'docker run --detach --name stellar --publish 8080:8080 stellar/quickstart:latest'\")"
					"chain.wait_for_open_port(8080)"
					
					"initial_balance=10000"
					"bootstrap_public_key=chain.succeed(\"nu -c 'stellar keys generate bootstrap --network local --fund | lines | find \\\"Public Key\\\" | parse \\\"Public Key: {key}\\\" | get key.0'\").strip()"
					
					"relay_public_key=chain.succeed(\"nu -c 'stellar keys generate relay --network local --fund --output json | from json | get public_key'\").strip()"
					"client_public_key=chain.succeed(\"nu -c 'stellar keys generate client --network local --fund --output json | from json | get public_key'\").strip()"
					"server_public_key=chain.succeed(\"nu -c 'stellar keys generate server --network local --fund --output json | from json | get public_key'\").strip()"

					"chain.succeed(f\"nu -c 'stellar account address {bootstrap_public_key} --rpc-url http://localhost:8080 | get balances | where asset_type == \\\"native\\\" | get balance | into float | $in == {initial_balance}'\")"
					"chain.succeed(f\"nu -c 'stellar account address {relay_public_key} --rpc-url http://localhost:8080 | get balances | where asset_type == \\\"native\\\" | get balance | into float | $in == {initial_balance}'\")"
					"chain.succeed(f\"nu -c 'stellar account address {client_public_key} --rpc-url http://localhost:8080 | get balances | where asset_type == \\\"native\\\" | get balance | into float | $in == {initial_balance}'\")"
					"chain.succeed(f\"nu -c 'stellar account address {server_public_key} --rpc-url http://localhost:8080 | get balances | where asset_type == \\\"native\\\" | get balance | into float | $in == {initial_balance}'\")"

					"bootstrap.start()"
					"bootstrap.wait_for_unit(\"network.target\")"
					"bootstrap.succeed(\"bootstrap --flag value > /dev/null 2>&1 &\")"
					
					"relay.start()"
					"relay.wait_for_unit(\"network.target\")"
					
					
					"client_router.wait_for_unit('network.target')"
					"client.wait_for_unit('network.target')"
					
					
					
					# "bootstrap.succeed('stellar --rpc-url http://${chain_wan_ip}:8080)"
					
					"bootstrap.execute('bootstrap > /var/log/bootstrap.log 2>&1 &')"
					"bootstrap.wait_for_open_port(8080)"
					"bootstrap.succeed('ss -uan | grep :4001')"
										
					"client.succeed('ping -c 2 192.168.2.254')"
					
					"bootstrap.succeed('ping -c 2 192.168.1.254')"

					# client -> bootstrap
					"client.succeed('ping -c 3 ${bootstrap_ip}')"

					# bootstrap -> client
					"bootstrap.fail('ping -c 3 -W 1 192.168.2.1')"
					
					"bootstrap.shutdown()"
					"chain.shutdown()"
					
				];
			};

			checks.default = pkgs.testers.runNixOSTest {
				name = "vm";

				# runs vm with access to internet so deps can be resolved...
				# ^sudo nix run .#vm -L --option sandbox false

				nodes.vm = { ... }: {
					system.stateVersion = "26.05";

					nix.settings.experimental-features = [
						"flakes"
						"nix-command"
					];

					virtualisation.cores = 4;
					virtualisation.diskSize = 40960;
					virtualisation.memorySize = 12288;
					virtualisation.docker.enable = true;

					boot.kernelPackages = pkgs.linuxPackages_latest;
					boot.kernel.sysctl."net.core.rmem_max" = 2500000;
					boot.kernel.sysctl."net.core.wmem_max" = 2500000;
					boot.kernel.sysctl."net.ipv4.ip_forward" = 1;
					boot.kernel.sysctl."net.ipv4.conf.all.forwarding" = 1;
					boot.kernel.sysctl."net.ipv4.conf.all.rp_filter" = 0;
					boot.kernel.sysctl."net.ipv4.conf.default.rp_filter" = 0;
					boot.kernelModules = [
						"br_netfilter"
					];

					networking.firewall.enable = true;
					networking.firewall.checkReversePath = false;
					networking.firewall.extraCommands = ''
						iptables -A FORWARD -i br-+ -o br-+ -j ACCEPT
						iptables -A FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
					'';

					systemd.network.networks."10-unmanaged-docker".matchConfig.Name = [
						"docker0"
						"veth*"
						"br-*"
					];
					systemd.network.networks."10-unmanaged-docker".linkConfig.Unmanaged = "yes";

					networking.useDHCP = true;
					networking.useNetworkd = true;
					networking.dhcpcd.enable = false;
					networking.dhcpcd.extraConfig = ''
				    	denyinterfaces veth*
				  	'';

					environment.etc."workspace".source = ./.;
					environment.systemPackages = [
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
						pkgs.gcc
						pkgs.protobuf
						pkgs.docker

						config.packages.bootstrap
						config.packages.relay
						config.packages.client
						config.packages.server
						config.packages.maliciousBootstrap
						config.packages.maliciousRelay
						config.packages.maliciousClient
						config.packages.maliciousServer
					];
				};

				testScript = pkgs.lib.concatLines [
					"vm.start()"
					"vm.wait_for_unit('docker.service')"
					"vm.succeed('cp -rL /etc/workspace /root/workspace')"
					"vm.succeed('chmod -R u+w /root/workspace')"
					"vm.succeed('export CARGO_TARGET_DIR=/var/tmp/cargo-target && cd /root/workspace && cargo test --jobs 1 --package node --no-default-features -- --nocapture')"
				];
			};
			
			packages.bootstrap = mkNode "bootstrap";
			packages.relay = mkNode "relay";
			packages.client = mkNode "client";
			packages.server = mkNode "server";
			packages.maliciousBootstrap = mkNode "malicious_bootstrap";
			packages.maliciousRelay = mkNode "malicious_relay";
			packages.maliciousClient = mkNode "malicious_client";
			packages.maliciousServer = mkNode "malicious_server";

			packages.stellar_testnet_image = pkgs.dockerTools.pullImage {
				imageName = "stellar/quickstart";
				imageDigest = "sha256:89d4990f8147956011f4090d5d125f7eb4604c6df3ad50289b55082ff1cb5217";
				sha256 = "sha256-kI/3/QW4hAwfMhDQKfyFRtWA1PDHhn8odMl/GG1hMxU=";
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
		};
	};
}
