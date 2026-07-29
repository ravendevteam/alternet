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

			mk_node = role: pkgs.rustPlatform.buildRustPackage {
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

			mk_soroban_contract = pname: pkgs.stdenv.mkDerivation rec {
				RUSTFLAGS = "-A warnings";

				inherit pname;

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

				nodes.chain.virtualisation.diskSize = 8000;
				nodes.chain.virtualisation.memorySize = 2500;
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
					"chain.wait_until_succeeds(\"curl -s -X POST -H 'Content-Type: application/json' -d '{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"method\\\":\\\"getHealth\\\"}' http://localhost:8080/soroban/rpc | grep -q 'unhealthy\\|healthy'\")"
					"chain.wait_until_succeeds(\"curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/friendbot | grep -q '400'\")"

					"chain.succeed(\"sleep 5\")"

					"chain.succeed(\"nu -c 'stellar keys generate bootstrap --network local'\")"
					"chain.succeed(\"nu -c 'stellar keys generate relay --network local'\")"
					"chain.succeed(\"nu -c 'stellar keys generate client --network local'\")"
					"chain.succeed(\"nu -c 'stellar keys generate server --network local'\")"

					"chain.succeed(\"nu -c 'stellar keys fund bootstrap'\")"
					"chain.succeed(\"nu -c 'stellar keys fund relay'\")"
					"chain.succeed(\"nu -c 'stellar keys fund client'\")"
					"chain.succeed(\"nu -c 'stellar keys fund server'\")"

					"bootstrap_public_key=chain.succeed(\"nu -c 'stellar keys public-key bootstrap'\").strip()"
					"bootstrap_secret_key=chain.succeed(\"nu -c 'stellar keys secret bootstrap'\").strip()"

					"relay_public_key=chain.succeed(\"nu -c 'stellar keys public-key relay'\").strip()"
					"relay_secret_key=chain.succeed(\"nu -c 'stellar keys secret relay'\").strip()"

					"client_public_key=chain.succeed(\"nu -c 'stellar keys public-key client'\").strip()"
					"client_secret_key=chain.succeed(\"nu -c 'stellar keys secret client'\").strip()"

					"server_public_key=chain.succeed(\"nu -c 'stellar keys public-key server'\").strip()"
					"server_secret_key=chain.succeed(\"nu -c 'stellar keys secret server'\").strip()"

					"bootstrap.start()"
					"bootstrap.wait_for_unit(\"network.target\")"
					# "bootstrap.succeed(\"bootstrap --flag value > /dev/null 2>&1 &\")"

					"bootstrap.succeed(\"stellar network add local --rpc-url http://192.168.1.3:8080/soroban/rpc --network-passphrase \"Standalone Network ; February 2017\"\")"
					"bootstrap.succeed(F\"echo {bootstrap_secret_key} | stellar keys add admin --secret-key\")"


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

			packages.bootstrap = mk_node "bootstrap";
			packages.relay = mk_node "relay";
			packages.client = mk_node "client";
			packages.server = mk_node "server";
			packages.maliciousBootstrap = mk_node "malicious_bootstrap";
			packages.maliciousRelay = mk_node "malicious_relay";
			packages.maliciousClient = mk_node "malicious_client";
			packages.maliciousServer = mk_node "malicious_server";

			packages.soroban_mock_dns = mk_soroban_contract "soroban_mock_dns";
			packages.soroban_mock_erc_20 = mk_soroban_contract "soroban_mock_erc_20";
			packages.soroban_mock_nft = mk_soroban_contract "soroban_mock_nft";

			packages.soroban_mock_dns_e2e = pkgs.testers.runNixOSTest {
				name = "main";
				
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

					networking.useDHCP = true;
					networking.useNetworkd = true;
					networking.dhcpcd.enable = false;
					networking.dhcpcd.extraConfig = ''
				    	denyinterfaces veth*
				  	'';

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

						config.packages.stellar
					];
				};
				
				testScript = pkgs.lib.concatLines [
					"vm.start()"
										
					"vm.wait_for_unit('network.target')"
					"vm.wait_for_unit('docker.service')"
					
					"vm.succeed(\"nu -c 'docker load --input ${config.packages.stellar_testnet_image}'\")"
					"vm.succeed(\"nu -c 'docker run --detach --name stellar --publish 8000:8000 stellar/quickstart:latest --local'\")"
					
					"vm.wait_for_open_port(8000)"
					
					"vm.wait_until_succeeds(\"curl -s -X POST -H 'Content-Type: application/json' -d '{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"method\\\":\\\"getHealth\\\"}' http://localhost:8000/soroban/rpc | grep -q 'unhealthy\\|healthy'\")"
					"vm.wait_until_succeeds(\"curl -s -o /dev/null -w '%{http_code}' http://localhost:8000/friendbot | grep -q '400'\")"
					
					"vm.succeed('sleep 5')"
					
					"vm.succeed(\"nu -c 'stellar network add local --rpc-url http://localhost:8000/soroban/rpc --network-passphrase \\\"Standalone Network ; February 2017\\\"'\")"
					"vm.succeed(\"nu -c 'stellar network use local'\")"
					
					"vm.succeed(\"nu -c 'stellar keys generate deployer'\")"
					"vm.succeed(\"nu -c 'stellar keys generate user_a'\")"
					"vm.succeed(\"nu -c 'stellar keys generate user_b'\")"
					
					"vm.succeed(\"nu -c 'stellar keys fund deployer'\")"
					"vm.succeed(\"nu -c 'stellar keys fund user_a'\")"
					"vm.succeed(\"nu -c 'stellar keys fund user_b'\")"
					
					"tkn_public_key = vm.succeed(\"nu -c 'stellar contract deploy --wasm ${config.packages.soroban_mock_erc_20}/lib/soroban_mock_erc_20.wasm --source deployer --network local'\").strip()"
					"nft_public_key = vm.succeed(\"nu -c 'stellar contract deploy --wasm ${config.packages.soroban_mock_nft}/lib/soroban_mock_nft.wasm --source deployer --network local'\").strip()"
					
					"vm.succeed(\"nu -c 'stellar contract invoke --id \" + tkn_public_key + \" --source deployer --network local -- configure --admin deployer --name \\\"Mock Token\\\" --symbol \\\"MCK\\\" --decimals \\\"2\\\" --initial_mint \\\"10000000\\\"'\")"
					"vm.succeed(\"nu -c 'stellar contract invoke --id \" + nft_public_key + \" --source deployer --network local -- configure --admin deployer --name \\\"Namespace\\\" --symbol \\\"NSP\\\"'\")"
					
					"dns_public_key = vm.succeed(\"nu -c 'stellar contract deploy --wasm ${config.packages.soroban_mock_dns}/lib/soroban_mock_dns.wasm --source deployer --network local'\").strip()"
					
					"vm.succeed(\"nu -c 'stellar contract invoke --id \" + dns_public_key + \" --source deployer --network local -- configure --tkn \" + tkn_public_key + \" --nft \" + nft_public_key + \" --mint_min_fee 10000 --mint_max_fee 90000 --renew_min_fee 10000 --renew_max_fee 90000 --harberger_tax_rate 500 --target_traffic 50000'\")"
				
					"deployer_addr = vm.succeed(\"nu -c 'stellar keys address deployer'\").strip()"
					"user_a_addr = vm.succeed(\"nu -c 'stellar keys address user_a'\").strip()"

					"vm.succeed(\"nu -c 'stellar contract invoke --id \" + tkn_public_key + \" --source deployer --network local -- transfer --sender \" + deployer_addr + \" --recipient \" + user_a_addr + \" --amount 50000'\")"
					"vm.succeed(\"nu -c 'stellar contract invoke --id \" + dns_public_key + \" --source user_a --network local -- mint --account \" + user_a_addr + \" --domain \\\"alternet\\\"'\")"

					"vm.succeed(\"nu -c 'stellar contract invoke --id \" + dns_public_key + \" --source user_a --network local -- renew --domain \\\"alternet\\\"'\")"
					
					"vm.shutdown()"
				];
			};
			
			packages.soroban_mock_nft_e2e = pkgs.testers.runNixOSTest {
				name = "main";

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

					networking.useDHCP = true;
					networking.useNetworkd = true;
					networking.dhcpcd.enable = false;
					networking.dhcpcd.extraConfig = ''
				    	denyinterfaces veth*
				  	'';

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

						config.packages.stellar
					];
				};

				testScript = pkgs.lib.concatLines [
					"vm.start()"

					"vm.wait_for_unit('network.target')"
					"vm.wait_for_unit('docker.service')"

					"vm.succeed(\"nu -c 'docker load --input ${config.packages.stellar_testnet_image}'\")"
					"vm.succeed(\"nu -c 'docker run --detach --name stellar --publish 8000:8000 stellar/quickstart:latest --local'\")"

					"vm.wait_for_open_port(8000)"

					"vm.wait_until_succeeds(\"curl -s -X POST -H 'Content-Type: application/json' -d '{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"method\\\":\\\"getHealth\\\"}' http://localhost:8000/soroban/rpc | grep -q 'unhealthy\\|healthy'\")"
					"vm.wait_until_succeeds(\"curl -s -o /dev/null -w '%{http_code}' http://localhost:8000/friendbot | grep -q '400'\")"

					"vm.succeed('sleep 5')"

					"vm.succeed(\"nu -c 'stellar network add local --rpc-url http://localhost:8000/soroban/rpc --network-passphrase \\\"Standalone Network ; February 2017\\\"'\")"
					"vm.succeed(\"nu -c 'stellar network use local'\")"

					"vm.succeed(\"nu -c 'stellar keys generate deployer'\")"
					"vm.succeed(\"nu -c 'stellar keys fund deployer'\")"
					"vm.succeed(\"nu -c 'stellar keys generate alice'\")"
					"vm.succeed(\"nu -c 'stellar keys fund alice'\")"

					"deployer_public_key = vm.succeed(\"nu -c 'stellar keys address deployer'\").strip()"
					
					"alice_public_key = vm.succeed(\"nu -c 'stellar keys address alice'\").strip()"

					"address = vm.succeed(\"nu -c 'stellar contract deploy --wasm ${config.packages.soroban_mock_nft}/lib/soroban_mock_nft.wasm --source deployer --network local'\").strip()"

					"vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- configure --admin deployer --name \\\"Mock Domain NFT\\\" --symbol \\\"MDN\\\"'\")"

					"assert \"Mock Domain NFT\" in vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- name'\")"
					"assert \"MDN\" in vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- symbol'\")"

					"vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- mint --owner deployer --domain \\\"stellar.stellar\\\"'\")"
					
					"assert deployer_public_key in vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- owner_of --domain \\\"stellar.stellar\\\"'\")"

					"vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- transfer --sender deployer --recipient {alice_public_key} --domain \\\"stellar.stellar\\\"'\")"

					"assert alice_public_key in vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- owner_of --domain \\\"stellar.stellar\\\"'\")"
					
					"vm.shutdown()"
				];
			};

			packages.soroban_mock_erc_20_e2e = pkgs.testers.runNixOSTest {
				name = "test";

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

					networking.useDHCP = true;
					networking.useNetworkd = true;
					networking.dhcpcd.enable = false;
					networking.dhcpcd.extraConfig = ''
				    	denyinterfaces veth*
				  	'';

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

						config.packages.stellar
					];
				};

				testScript = pkgs.lib.concatLines [
					"vm.start()"

					"vm.wait_for_unit('network.target')"
					"vm.wait_for_unit('docker.service')"

					"vm.succeed(\"nu -c 'docker load --input ${config.packages.stellar_testnet_image}'\")"
					"vm.succeed(\"nu -c 'docker run --detach --name stellar --publish 8000:8000 stellar/quickstart:latest --local'\")"

					"vm.wait_for_open_port(8000)"

					"vm.wait_until_succeeds(\"curl -s -X POST -H 'Content-Type: application/json' -d '{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"method\\\":\\\"getHealth\\\"}' http://localhost:8000/soroban/rpc | grep -q 'unhealthy\\|healthy'\")"
					"vm.wait_until_succeeds(\"curl -s -o /dev/null -w '%{http_code}' http://localhost:8000/friendbot | grep -q '400'\")"

					"vm.succeed('sleep 5')"

					"vm.succeed(\"nu -c 'stellar network add local --rpc-url http://localhost:8000/soroban/rpc --network-passphrase \\\"Standalone Network ; February 2017\\\"'\")"
					"vm.succeed(\"nu -c 'stellar network use local'\")"
					"vm.succeed(\"nu -c 'stellar keys generate deployer'\")"
					"vm.succeed(\"nu -c 'stellar keys fund deployer'\")"

					"address = vm.succeed(\"nu -c 'stellar contract deploy --wasm ${config.packages.soroban_mock_erc_20}/lib/soroban_mock_erc_20.wasm --source deployer --network local'\").strip()"

					"vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- configure --admin deployer --name \\\"Mock Token\\\" --symbol \\\"MCK\\\" --decimals \\\"18\\\" --initial_mint \\\"1000000000000000000\\\"'\")"

					"assert \"Mock Token\" in vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- name'\")"
					"assert \"MCK\" in vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- symbol'\")"
					"assert \"18\" in vm.succeed(F\"nu -c 'stellar contract invoke --id {address} --source deployer --network local -- decimals'\")"

					"vm.shutdown()"
				];
			};

			packages.stellar_testnet_image = import ./nix/stellar_testnet_image.nix {
				inherit pkgs;
			};

			packages.stellar = import ./nix/stellar.nix {
				inherit pkgs;
				inherit system;
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
		};
	};
}
