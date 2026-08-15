{ self, inputs, ... }: {
	perSystem = { pkgs, inputs', ... }:
	let
		crane = (inputs.crane.mkLib pkgs).overrideToolchain inputs'.fenix.packages.stable.toolchain;

		node_args.src = self;
		node_args.pname = "node";
		node_args.version = "0.1.0";
		node_args.doCheck = false;
		node_args.nativeBuildInputs = [
			pkgs.pkg-config
			pkgs.protobuf
		];
		node_args.buildInputs = [
			pkgs.openssl
		];
		node_extra_args.cargoExtraArgs = "--package node --no-default-features";
		node_artifacts = crane.buildDepsOnly (node_args // node_extra_args);
		mk_node = role: crane.buildPackage (node_args // {
			pname = role;
			cargoArtifacts = node_artifacts;
			cargoExtraArgs = "--package node --bin ${role} --features ${role} --no-default-features";
		});
	in {
		packages.bootstrap = mk_node "bootstrap";
		packages.relay = mk_node "relay";
		packages.client = mk_node "client";
		packages.server = mk_node "server";
		packages.malicious_bootstrap = mk_node "malicious_bootstrap";
		packages.malicious_relay = mk_node "malicious_relay";
		packages.malicious_client = mk_node "malicious_client";
		packages.malicious_server = mk_node "malicious_server";
	};
}
