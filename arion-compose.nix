# needs a deterministic address seed, more experimentation required

# . attempt with found public key for seed 0..
{ pkgs, ... }:
let
	mkNode = role: pkgs.rustPlatform.buildRustPackage {
		RUSTFLAGS = "-Awarnings";
		pname = "node_${role}";
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
	bootstrap = mkNode "bootstrap";
	relay = mkNode "relay";
	client = mkNode "client";
	server = mkNode "server";
in {
	config.project.name = "milestone_2";
	config.networks.wan = {};
	config.networks.client_lan.internal = true;
	config.networks.server_lan.internal = true;
	
	config.services.bootstrap.service.useHostStore = true;
	config.services.bootstrap.service.privileged = true;
	config.services.bootstrap.service.capabilities.NET_ADMIN = true;
	config.services.bootstrap.service.networks = [
		"wan"
	];
	config.services.bootstrap.service.ports = [
		"4000:4001/udp"
		"8000:8080"
	];
	config.services.bootstrap.service.entrypoint = ''
		${bootstrap}/bin/bootstrap --seed 0000000000000000000000000000000000000000000000000000000000000000
	'';

	config.services.relay.service.useHostStore = true;
	config.services.relay.service.privileged = true;
	config.services.relay.service.capabilities.NET_ADMIN = true;
	config.services.relay.service.networks = [
		"wan"
	];
	config.services.relay.service.ports = [
		"4001:4001/udp"
		"8001:8080"
	];
	config.services.relay.service.entrypoint = ''
		${relay}/bin/relay --dial /dns4/bootstrap/udp/4001/quic-v1/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN
	'';

	config.services.client_router.service.image = "alpine:latest";
	config.services.client_router.service.privileged = true;
	config.services.client_router.service.networks = [
		"wan"
		"client_lan"
	];
	config.services.client_router.service.entrypoint = ''
		sh -c "
			apk add --no-cache iptables

			sysctl -w net.ipv4.ip_forward=1

			iptables -t nat -A POSTROUTING -o eth0 -p udp -j MASQUERADE --random-fully
			iptables -A FORWARD -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
			iptables -A FORWARD -d bootstrap -p udp --dport 4001 -j ACCEPT
			iptables -A FORWARD -d relay -p udp --dport 4001 -j ACCEPT
			iptables -A FORWARD -d server_router -p udp --dport 4001 -j DROP
			iptables -A FORWARD -i eth1 -j ACCEPT
			iptables -P FORWARD DROP

			sleep infinity
		"
	'';

	config.services.server_router.service.image = "alpine:latest";
	config.services.server_router.service.privileged = true;
	config.services.server_router.service.networks = [
		"wan"
		"server_lan"
	];
	config.services.server_router.service.entrypoint = ''
		sh -c "
			apk add --no-cache iptables

			sysctl -w net.ipv4.ip_forward=1

			iptables -t nat -A POSTROUTING -o eth0 -p udp -j MASQUERADE --random-fully
			iptables -A FORWARD -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
			iptables -A FORWARD -d bootstrap -p udp --dport 4001 -j ACCEPT
			iptables -A FORWARD -d relay -p udp --dport 4001 -j ACCEPT
			iptables -A FORWARD -s client_router -p udp --dport 4001 -j DROP
			iptables -A FORWARD -i eth1 -j ACCEPT
			iptables -P FORWARD DROP

			sleep infinity
		"
	'';

	config.services.client.service.useHostStore = true;
	config.services.client.service.privileged = true;
	config.services.client.service.capabilities.NET_ADMIN = true;
	config.services.client.service.networks = [
		"client_lan"
	];
	config.services.client.service.ports = [
		"4002:4001/udp"
		"8002:8080"
	];
	config.services.client.service.entrypoint = ''
		${client}/bin/client --dial /dns4/bootstrap/udp/4001/quic-v1/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN
	'';

	config.services.server.service.useHostStore = true;
	config.services.server.service.privileged = true;
	config.services.server.service.capabilities.NET_ADMIN = true;
	config.services.server.service.networks = [
		"server_lan"
	];
	config.services.server.service.ports = [
		"4003:4001/udp"
		"8003:8080"
	];
	config.services.server.service.entrypoint = ''
		${server}/bin/server --dial /dns4/bootstrap/udp/4001/quic-v1/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN
	'';
}
