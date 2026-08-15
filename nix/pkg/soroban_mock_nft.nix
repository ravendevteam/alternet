{ self, ... }: {
	perSystem = { pkgs, config, ... }: {
		packages.soroban_mock_nft = (import ../lib/mk_soroban_contract self pkgs config "soroban_mock_nft");
	};
}
