{
	perSystem = { pkgs, config, ... }: {
		checks.e2e =
		let
			wan = 1;
			isp_wan_ip = "192.168.1.254";
			chain_wan_ip = "192.168.1.3";
			bootstrap_ip = "192.168.1.1";
			relay_ip = "192.168.1.2";
			client_lan = 2;
			client_ip = "192.168.2.1";
			client_router_wan_eth = "eth1";
			client_router_lan_eth = "eth2";
			client_router_wan_ip = "192.168.1.252";
			client_router_lan_ip = "192.168.2.254";
			server_lan = 3;
			server_router_wan_ip = "192.168.1.253";
		in pkgs.testers.runNixOSTest {
			name = "e2e";

			nodes.isp.system.stateVersion = "26.05";

			nodes.isp.virtualisation.vlans = [
				wan
			];

			nodes.isp.boot.kernel.sysctl."net.ipv4.ip_forward" = 1;

			nodes.isp.networking.useDHCP = false;
			nodes.isp.networking.interfaces.eth1.ipv4.addresses = [{ address = isp_wan_ip; prefixLength = 24; }];
			nodes.isp.networking.interfaces.eth1.ipv4.routes = [
				{ address = "192.168.2.0"; prefixLength = 24; via = client_router_wan_ip; }
				{ address = "192.168.3.0"; prefixLength = 24; via = server_router_wan_ip; }
			];

			nodes.chain.system.stateVersion = "26.05";

			nodes.chain.virtualisation.diskSize = 8000;
			nodes.chain.virtualisation.memorySize = 2500;
			nodes.chain.virtualisation.docker.enable = true;
			nodes.chain.virtualisation.docker.autoPrune.enable = true;
			nodes.chain.virtualisation.vlans = [
				wan
			];

			nodes.chain.networking.useDHCP = false;
			nodes.chain.networking.defaultGateway = isp_wan_ip;
			nodes.chain.networking.interfaces.eth1.ipv4.addresses = [{ address = chain_wan_ip; prefixLength = 24; }];
			nodes.chain.networking.firewall.allowedTCPPorts = [
				8080
			];

			nodes.chain.environment.systemPackages = [
				pkgs.nushell
				pkgs.docker

				config.packages.stellar
			];

			nodes.bootstrap.system.stateVersion = "26.05";

			nodes.bootstrap.virtualisation.vlans = [
				wan
			];

			nodes.bootstrap.networking.useDHCP = false;
			nodes.bootstrap.networking.defaultGateway = isp_wan_ip;
			nodes.bootstrap.networking.interfaces.eth1.ipv4.addresses = [{ address = bootstrap_ip; prefixLength = 24; }];

			nodes.bootstrap.environment.systemPackages = [
				pkgs.nushell

				config.packages.bootstrap
				config.packages.stellar
			];

			nodes.relay.system.stateVersion = "26.05";

			nodes.relay.virtualisation.vlans = [
				wan
			];

			nodes.relay.environment.systemPackages = [
				pkgs.nushell

				config.packages.relay
				config.packages.stellar
			];

			nodes.relay.networking.useDHCP = false;
			nodes.relay.networking.defaultGateway = isp_wan_ip;
			nodes.relay.networking.interfaces.eth1.ipv4.addresses = [{ address = relay_ip; prefixLength = 24; }];

			nodes.client.system.stateVersion = "26.05";

			nodes.client.virtualisation.vlans = [
				client_lan
			];

			nodes.client.networking.useDHCP = false;
			nodes.client.networking.defaultGateway = client_router_lan_ip;
			nodes.client.networking.interfaces.eth1.ipv4.addresses = [{ address = client_ip; prefixLength = 24; }];

			nodes.client_router.system.stateVersion = "26.05";

			nodes.client_router.virtualisation.vlans = [
				wan
				client_lan
			];

			nodes.client_router.boot.kernel.sysctl."net.ipv4.ip_forward" = 1;

			nodes.client_router.networking.useDHCP = false;
			nodes.client_router.networking.firewall.enable = true;
			nodes.client_router.networking.firewall.allowPing = true;
			nodes.client_router.networking.firewall.extraCommands = ''
				iptables -t nat -A POSTROUTING -o ${client_router_wan_eth} -p udp -j MASQUERADE --random-fully
				iptables -A FORWARD -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
				iptables -A FORWARD -i ${client_router_lan_eth} -o ${client_router_wan_eth} -j ACCEPT
				iptables -A FORWARD -d ${bootstrap_ip} -p udp --dport 4001 -j ACCEPT
				iptables -A FORWARD -d ${relay_ip} -p udp --dport 4001 -j ACCEPT
				iptables -A FORWARD -d ${server_router_wan_ip} -p udp --dport 4001 -j DROP
				iptables -P FORWARD DROP

				tc qdisc add dev ${client_router_wan_eth} root netem delay 100ms 10ms loss 1%
			'';

			nodes.client_router.networking.interfaces.eth1.ipv4.addresses = [{ address = client_router_wan_ip; prefixLength = 24; }];
			nodes.client_router.networking.interfaces.eth2.ipv4.addresses = [{ address = client_router_lan_ip; prefixLength = 24; }];

			# --- call grpc to interact with the nodes within the vm
			# grpcurl localhost:8080 list
			# grpcurl localhost:8080 list your.package.ServiceName
			# grpcurl -d '{"field_name": "value"}' localhost:8080 your.package.ServiceName/MethodName

			testScript = pkgs.lib.concatLines [
				"isp.start()"
				"isp.wait_for_unit(\"network.target\")"

				"chain.start()"

				"chain.wait_for_unit(\"network.target\")"
				"chain.wait_for_unit(\"docker.service\")"

				"chain.succeed(\"nu -c 'docker load --input ${config.packages.stellar_testnet_image}'\")"
				"chain.succeed(\"nu -c 'docker run --detach --name stellar --publish 8080:8080 stellar/quickstart:latest'\")"

				"chain.wait_for_open_port(8080)"
				"chain.wait_until_succeeds(\"curl -s -X POST -H 'Content-Type: application/json' -d '{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":1,\\\"method\\\":\\\"getHealth\\\"}' http://localhost:8080/soroban/rpc | grep -q 'unhealthy\\|healthy'\")"
				"chain.wait_until_succeeds(\"curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/friendbot | grep -q '400'\")"

				"chain.succeed(\"sleep 5\")"

				"chain.succeed(\"nu -c 'stellar keys generate bootstrap --network local'\")"
				"chain.succeed(\"nu -c 'stellar keys generate relay --network local'\")"
				"chain.succeed(\"nu -c 'stellar keys generate client --network local'\")"
				"chain.succeed(\"nu -c 'stellar keys generate server --network local'\")"

				"chain.succeed(\"nu -c 'stellar keys fund bootstrap'\")"
				"chain.succeed(\"nu -c 'stellar keys fund relay'\")"
				"chain.succeed(\"nu -c 'stellar keys fund client'\")"
				"chain.succeed(\"nu -c 'stellar keys fund server'\")"

				"bootstrap_public_key=chain.succeed(\"nu -c 'stellar keys public-key bootstrap'\").strip()"
				"bootstrap_secret_key=chain.succeed(\"nu -c 'stellar keys secret bootstrap'\").strip()"

				"relay_public_key=chain.succeed(\"nu -c 'stellar keys public-key relay'\").strip()"
				"relay_secret_key=chain.succeed(\"nu -c 'stellar keys secret relay'\").strip()"

				"client_public_key=chain.succeed(\"nu -c 'stellar keys public-key client'\").strip()"
				"client_secret_key=chain.succeed(\"nu -c 'stellar keys secret client'\").strip()"

				"server_public_key=chain.succeed(\"nu -c 'stellar keys public-key server'\").strip()"
				"server_secret_key=chain.succeed(\"nu -c 'stellar keys secret server'\").strip()"

				"bootstrap.start()"
				"bootstrap.wait_for_unit(\"network.target\")"
				# "bootstrap.succeed(\"bootstrap --flag value > /dev/null 2>&1 &\")"

				"bootstrap.succeed(\"stellar network add local --rpc-url http://192.168.1.3:8080/soroban/rpc --network-passphrase \"Standalone Network ; February 2017\"\")"
				"bootstrap.succeed(F\"echo {bootstrap_secret_key} | stellar keys add admin --secret-key\")"


				"relay.start()"
				"relay.wait_for_unit(\"network.target\")"


				"client_router.wait_for_unit('network.target')"
				"client.wait_for_unit('network.target')"



				# "bootstrap.succeed('stellar --rpc-url http://${chain_wan_ip}:8080)"

				"bootstrap.execute('bootstrap > /var/log/bootstrap.log 2>&1 &')"
				"bootstrap.wait_for_open_port(8080)"
				"bootstrap.succeed('ss -uan | grep :4001')"

				"client.succeed('ping -c 2 192.168.2.254')"

				"bootstrap.succeed('ping -c 2 192.168.1.254')"

				# client -> bootstrap
				"client.succeed('ping -c 3 ${bootstrap_ip}')"

				# bootstrap -> client
				"bootstrap.fail('ping -c 3 -W 1 192.168.2.1')"

				"bootstrap.shutdown()"
				"chain.shutdown()"

			];
		};
	};
}
