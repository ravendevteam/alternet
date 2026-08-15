{ self, ... }: {
	perSystem = { pkgs, config, ... }: {
		packages.soroban_mock_nft = pkgs.lib.mkForce (import ../lib/mk_soroban_contract.nix self pkgs config "soroban_mock_nft");
	};
}
