{
	perSystem = { pkgs, config, ... }:
	let
		name = "main";

		no_firewall_extra_commands = "";
		localhost = "0.0.0.0";
		localhost_grpc_endpoint_port = "8080";
		localhost_grpc_endpoint = "${localhost}:${localhost_grpc_endpoint_port}";
		schema = ../../app/node/proto/an.proto;
		system_state_version = "26.05";

		mk_grpc_call = node: endpoint: "last = json.loads(${node}.succeed(\"grpcurl -plaintext -import-path / -proto ${schema} -d '{}' ${localhost_grpc_endpoint} an.Node/${endpoint}\"))";

		mk_check = extra_commands: pkgs.testers.runNixOSTest {
			name = name;
			nodes.router.system.stateVersion = system_state_version;
			nodes.router.virtualisation.vlans = [1 2];
			nodes.router.boot.kernel.sysctl."net.ipv4.ip_forward" = 1;

			nodes.router.networking.useDHCP = false;
			nodes.router.networking.firewall.enable = true;
			nodes.router.networking.firewall.extraCommands = extra_commands;

			nodes.router.networking.interfaces.eth1.ipv4.addresses = [{ address = "192.168.1.254"; prefixLength = 24; }];
			nodes.router.networking.interfaces.eth2.ipv4.addresses = [{ address = "192.168.2.254"; prefixLength = 24; }];

			nodes.router.environment.systemPackages = [
				pkgs.iptables
				pkgs.iproute2
			];

			nodes.bootstrap.system.stateVersion = system_state_version;
			nodes.bootstrap.virtualisation.vlans = [1];
			nodes.bootstrap.networking.useDHCP = false;
			nodes.bootstrap.networking.interfaces.eth1.ipv4.addresses = [{ address = "192.168.1.1"; prefixLength = 24; }];
			nodes.bootstrap.networking.firewall.allowedTCPPorts = [4001 4002];
			nodes.bootstrap.networking.firewall.allowedUDPPorts = [4001];
			nodes.bootstrap.networking.nameservers = ["1.1.1.1" "8.8.8.8"];
			nodes.bootstrap.networking.defaultGateway = "192.168.1.254";

			nodes.bootstrap.environment.systemPackages = [
				pkgs.iproute2
				pkgs.nettools
				pkgs.grpcurl

				config.packages.bootstrap
			];

			nodes.bootstrap.systemd.services.bootstrap.wantedBy = ["multi-user.target"];
			nodes.bootstrap.systemd.services.bootstrap.after = ["network.target"];
			nodes.bootstrap.systemd.services.bootstrap.serviceConfig.Restart = "on-failure";
			nodes.bootstrap.systemd.services.bootstrap.serviceConfig.ExecStart = "${config.packages.bootstrap}/bin/bootstrap";

			nodes.client.system.stateVersion = system_state_version;
			nodes.client.virtualisation.vlans = [2];
			nodes.client.networking.useDHCP = false;
			nodes.client.networking.interfaces.eth1.ipv4.addresses = [{ address = "192.168.2.1"; prefixLength = 24; }];
			nodes.client.networking.nameservers = ["1.1.1.1" "8.8.8.8"];
			nodes.client.networking.defaultGateway = "192.168.2.254";

			nodes.client.environment.systemPackages = [
				pkgs.iproute2
				pkgs.nettools
				pkgs.grpcurl

				config.packages.client
			];

			nodes.client.systemd.services.client.wantedBy = ["multi-user.target"];
			nodes.client.systemd.services.client.after = ["network.target"];
			nodes.client.systemd.services.client.serviceConfig.Restart = "on-failure";
			nodes.client.systemd.services.client.serviceConfig.ExecStart = ''
				${config.packages.client}/bin/client \
					--dial /ip4/192.168.1.1/udp/4001/quic-v1 \
					--dial /ip4/192.168.1.1/tcp/4001 \
					--dial /ip4/192.168.1.1/tcp/4002/ws
			'';

			testScript = pkgs.lib.concatLines [
				"import json"

				"router.start()"
				"router.wait_for_unit(\"network.target\")"

				"bootstrap.start()"
				"bootstrap.wait_for_unit(\"network.target\")"
				"bootstrap.wait_for_unit(\"bootstrap.service\")"
				"bootstrap.wait_for_open_port(8080)"
				"bootstrap.wait_for_open_port(4001)"
				"bootstrap.wait_for_open_port(4002)"

				"client.start()"
				"client.wait_for_unit(\"network.target\")"
				"client.wait_for_unit(\"client.service\")"
				"client.wait_for_open_port(8080)"
				"client.wait_for_open_port(4001)"
				"client.wait_for_open_port(4002)"

				(mk_grpc_call "client" "Peers")

				"assert len(last.get(\"peers\", [])) >= 1, \"expected at least 1 peer, got none\""
			];
		};
	in {
		checks.transport = mk_check no_firewall_extra_commands;
		
		checks.transport_without_tcp = mk_check ''
			iptables -A FORWARD -p tcp --dport 4001 -j DROP
		'';

		checks.transport_without_quic = mk_check ''
			iptables -A FORWARD -p udp --dport 4001 -j DROP
		'';

		checks.transport_without_quic_and_tcp = mk_check ''
			iptables -A FORWARD -p udp --dport 4001 -j DROP
			iptables -A FORWARD -p tcp --dport 4001 -j DROP
		'';
	};
}
