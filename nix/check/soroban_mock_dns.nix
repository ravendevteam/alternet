{
	perSystem = { pkgs, config, ... }: {
		checks.soroban_mock_dns = pkgs.testers.runNixOSTest {
			name = "main";

			nodes.vm.system.stateVersion = "26.05";
			nodes.vm.nix.settings.experimental-features = ["flakes" "nix-command"];
			nodes.vm.virtualisation.cores = 4;
			nodes.vm.virtualisation.diskSize = 40960;
			nodes.vm.virtualisation.memorySize = 12288;
			nodes.vm.virtualisation.docker.enable = true;

			nodes.vm.networking.useDHCP = true;
			nodes.vm.networking.useNetworkd = true;
			nodes.vm.networking.dhcpcd.enable = false;
			nodes.vm.networking.dhcpcd.extraConfig = ''
				denyinterfaces veth*
			'';

			nodes.vm.environment.systemPackages = [
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
	};
}