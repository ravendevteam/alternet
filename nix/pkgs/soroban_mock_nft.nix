{
	perSystem = { pkgs, config, ... }: {
		packages.soroban_mock_nft = (import ../lib/mk_soroban_contract pkgs config "soroban_mock_nft");
	};
}
