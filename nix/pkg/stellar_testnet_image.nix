{
	perSystem = { pkgs, ... }: {
		packages.stellar_testnet_image = pkgs.dockerTools.pullImage {
			imageName = "stellar/quickstart";
			imageDigest = "sha256:89d4990f8147956011f4090d5d125f7eb4604c6df3ad50289b55082ff1cb5217";
			sha256 = "sha256-kI/3/QW4hAwfMhDQKfyFRtWA1PDHhn8odMl/GG1hMxU=";
		};
	};
}
