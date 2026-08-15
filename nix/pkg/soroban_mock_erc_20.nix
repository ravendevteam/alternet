{ self, ... }: {
	perSystem = { pkgs, config, ... }: {
		packages.soroban_mock_erc_20 = (import ../lib/mk_soroban_contract self pkgs config "soroban_mock_erc_20");
	};
}
