{ self, ... }: {
	perSystem = { pkgs, config, ... }: {
		packages.soroban_mock_dns = pkgs.lib.mkForce (import ../lib/mk_soroban_contract.nix self pkgs config "soroban_mock_dns");
	};
}
