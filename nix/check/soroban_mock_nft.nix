{
	perSystem = { pkgs, config, ... }: {
		checks.soroban_mock_nft_e2e = pkgs.testers.runNixOSTest {
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
	};
}
