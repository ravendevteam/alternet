use image::Image as _;
use testcontainers::{ImageExt as _, runners::AsyncRunner};

mod proto {
    include!("../proto_target/an.rs");
}

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    let log_dir: std::path::PathBuf = std::path::PathBuf::new()
        .join("tests")
        .join("log")
        .join("dial_router");

    let node_image: testcontainers::GenericImage = image::node::Node::render().await?;

    let alpine_image: testcontainers::GenericImage = testcontainers::GenericImage::new("alpine", "latest");

    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults()?;

    let wan: e2e::network::Network = e2e::network::Network::new(&docker, format!("wan.{}", nanoid::nanoid!())).await?;
    let client_lan: e2e::network::Network = e2e::network::Network::new(&docker, format!("client_lan.{}", nanoid::nanoid!())).await?;
    let server_lan: e2e::network::Network = e2e::network::Network::new(&docker, format!("server_lan.{}", nanoid::nanoid!())).await?;

    let udp_port: u16 = 4001;
    let tcp_port: u16 = 8080;

    let bootstrap: testcontainers::ContainerAsync<_> = node_image
        .to_owned()
        .with_exposed_port(testcontainers::core::ContainerPort::Udp(udp_port))
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(tcp_port))
        .with_privileged(true)
        .with_container_name(format!("bootstrap.{}", nanoid::nanoid!()))
        .with_cmd(["./bootstrap"])
        .start()
        .await?;
    
    let bootstrap: e2e::container::Container<_> = e2e::container::Container::new(&docker, bootstrap);

    bootstrap.connect_to(&wan).await?;

    let bootstrap_ip: std::net::Ipv4Addr = bootstrap.ip(&wan).await?.ok_or(anyhow::anyhow!("no connection"))?;

    let bootstrap_grpc_host: url::Host = bootstrap.host().await?;
    let bootstrap_grpc_port: u16 = bootstrap.host_ipv4_port(tcp_port).await?;
    let bootstrap_grpc_endpoint: String = format!("http://{}:{}", bootstrap_grpc_host, bootstrap_grpc_port);

    let mut bootstrap_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(bootstrap_grpc_endpoint).await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let bootstrap_peer_id_request: proto::PeerIdRequest = proto::PeerIdRequest {

    };

    let bootstrap_peer_id_response: tonic::Response<_> = bootstrap_grpc.peer_id(bootstrap_peer_id_request).await?;
    let bootstrap_peer_id_response: proto::PeerIdResponse = bootstrap_peer_id_response.into_inner();
    let bootstrap_peer_id: String = bootstrap_peer_id_response.peer_id;

    let bootstrap_mu: libp2p::Multiaddr = format!("/ip4/{}/udp/4001/quic-v1/p2p/{}", bootstrap_ip, bootstrap_peer_id).parse()?;

    dbg!(&bootstrap_ip);
    dbg!(&bootstrap_mu);
    dbg!(&bootstrap_grpc_host);
    dbg!(&bootstrap_grpc_port);
    dbg!(&bootstrap_peer_id);

    let relay: testcontainers::ContainerAsync<_> = node_image
        .to_owned()
        .with_exposed_port(testcontainers::core::ContainerPort::Udp(udp_port))
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(tcp_port))
        .with_privileged(true)
        .with_container_name(format!("relay.{}", nanoid::nanoid!()))
        .with_cmd(["./relay", "--dial", &format!("{}", &bootstrap_mu)])
        .start()
        .await?;

    let relay: e2e::container::Container<_> = e2e::container::Container::new(&docker, relay);

    relay.connect_to(&wan).await?;

    let relay_ip: std::net::Ipv4Addr = relay.ip(&wan).await?.ok_or(anyhow::anyhow!("no connection"))?;
    
    let relay_grpc_host: url::Host = relay.host().await?;
    let relay_grpc_port: u16 = relay.host_ipv4_port(tcp_port).await?;
    let relay_grpc_endpoint: String = format!("http://{}:{}", relay_grpc_host, relay_grpc_port);

    let mut relay_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(relay_grpc_endpoint).await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let relay_peer_id_request: proto::PeerIdRequest = proto::PeerIdRequest {

    };

    let relay_peer_id_response: tonic::Response<_> = relay_grpc.peer_id(relay_peer_id_request).await?;
    let relay_peer_id_response: proto::PeerIdResponse = relay_peer_id_response.into_inner();
    let relay_peer_id: String = relay_peer_id_response.peer_id;

    dbg!(&relay_ip);
    dbg!(&relay_grpc_host);
    dbg!(&relay_grpc_port);
    dbg!(&relay_peer_id);

    let server_router: testcontainers::ContainerAsync<_> = alpine_image
        .to_owned()
        .with_container_name(format!("server_router.{}", nanoid::nanoid!()))
        .with_startup_timeout(std::time::Duration::from_mins(1))
        .with_privileged(true)
        .with_cmd(["sleep", "infinity"])
        .start()
        .await?;
    
    let server_router: e2e::container::Container<_> = e2e::container::Container::new(&docker, server_router);

    server_router.connect_to(&server_lan).await?;
    server_router.connect_to(&wan).await?;

    let server_router_lan_ip: std::net::Ipv4Addr = server_router.ip(&server_lan).await?.ok_or(anyhow::anyhow!("no connection"))?;
    let server_router_wan_ip: std::net::Ipv4Addr = server_router.ip(&wan).await?.ok_or(anyhow::anyhow!("no connection"))?;

    let server_router_lan_eth: usize = server_router.eth(&server_lan).await?;
    let server_router_lan_eth: String = format!("eth{}", server_router_lan_eth);
    let server_router_wan_eth: usize = server_router.eth(&wan).await?;
    let server_router_wan_eth: String = format!("eth{}", server_router_wan_eth);

    dbg!(&server_router_lan_ip);
    dbg!(&server_router_wan_ip);
    dbg!(&server_router_lan_eth);
    dbg!(&server_router_wan_eth);

    server_router.exec().args(vec!["apk", "add", "iptables"]).send().await?;
    server_router.exec().args(vec!["sysctl", "-w", "net.ipv4.ip_forward=1"]).send().await?;
    server_router.exec().args(vec!["iptables", "-t", "nat", "-A", "POSTROUTING", "-o", &server_router_wan_eth, "-j", "MASQUERADE"]).send().await?;
    server_router.exec().args(vec!["iptables", "-A", "FORWARD", "-i", &server_router_lan_eth, "-o", &server_router_wan_eth, "-j", "ACCEPT"]).send().await?;    
    server_router.exec().args(vec!["iptables", "-A", "FORWARD", "-m", "conntrack", "--ctstate", "ESTABLISHED,RELATED", "-j", "ACCEPT"]).send().await?;

    let server: testcontainers::ContainerAsync<_> = node_image
        .to_owned()
        .with_exposed_port(testcontainers::core::ContainerPort::Udp(udp_port))
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(tcp_port))
        .with_privileged(true)
        .with_container_name(format!("server.{}", nanoid::nanoid!()))
        .with_cmd(["./server", "--dial", &format!("{}", &bootstrap_mu)])
        .start()
        .await?;

    let server: e2e::container::Container<_> = e2e::container::Container::new(&docker, server);

    server.connect_to(&server_lan).await?;
    server.set_default_gateway(&server_router_lan_ip).await?;

    let server_ip: std::net::Ipv4Addr = server.ip(&server_lan).await?.ok_or(anyhow::anyhow!("no connection"))?;

    let server_grpc_host: url::Host = server.host().await?;
    let server_grpc_port: u16 = server.host_ipv4_port(tcp_port).await?;
    let server_grpc_endpoint: String = format!("http://{}:{}", server_grpc_host, server_grpc_port);

    let mut server_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(server_grpc_endpoint).await?;
    
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let server_peer_id_request: proto::PeerIdRequest = proto::PeerIdRequest {

    };

    let server_peer_id_response: tonic::Response<_> = server_grpc.peer_id(server_peer_id_request).await?;
    let server_peer_id_response: proto::PeerIdResponse = server_peer_id_response.into_inner();
    let server_peer_id: String = server_peer_id_response.peer_id;
    
    let server_mu_via_relay: libp2p::Multiaddr = format!("/ip4/{}/udp/4001/quic-v1/p2p/{}/p2p-circuit/p2p/{}", relay_ip, relay_peer_id, server_peer_id).parse()?;

    dbg!(&server_ip);
    dbg!(&server_grpc_host);
    dbg!(&server_grpc_port);
    dbg!(&server_peer_id);
    dbg!(&server_mu_via_relay);

    let client_router: testcontainers::ContainerAsync<_> = alpine_image
        .to_owned()
        .with_container_name(format!("client_router.{}", nanoid::nanoid!()))
        .with_startup_timeout(std::time::Duration::from_mins(1))
        .with_privileged(true)
        .with_cmd(["sleep", "infinity"])
        .start()
        .await?;

    let client_router: e2e::container::Container<_> = e2e::container::Container::new(&docker, client_router);

    client_router.connect_to(&client_lan).await?;
    client_router.connect_to(&wan).await?;
    
    let client_router_lan_ip: std::net::Ipv4Addr = client_router.ip(&client_lan).await?.ok_or(anyhow::anyhow!("no connection"))?;
    let client_router_wan_ip: std::net::Ipv4Addr = client_router.ip(&wan).await?.ok_or(anyhow::anyhow!("no connection"))?;

    let client_router_lan_eth: usize = client_router.eth(&client_lan).await?;
    let client_router_lan_eth: String = format!("eth{}", client_router_lan_eth);
    let client_router_wan_eth: usize = client_router.eth(&wan).await?;
    let client_router_wan_eth: String = format!("eth{}", client_router_wan_eth);

    dbg!(&client_router_lan_ip);
    dbg!(&client_router_wan_ip);
    dbg!(&client_router_lan_eth);
    dbg!(&client_router_wan_eth);

    client_router.exec().args(vec!["apk", "add", "iptables"]).send().await?;
    client_router.exec().args(vec!["sysctl", "-w", "net.ipv4.ip_forward=1"]).send().await?;
    client_router.exec().args(vec!["iptables", "-t", "nat", "-A", "POSTROUTING", "-o", &client_router_wan_eth, "-j", "MASQUERADE"]).send().await?;
    client_router.exec().args(vec!["iptables", "-A", "FORWARD", "-i", &client_router_lan_eth, "-o", &client_router_wan_eth, "-j", "ACCEPT"]).send().await?;    
    client_router.exec().args(vec!["iptables", "-A", "FORWARD", "-m", "conntrack", "--ctstate", "ESTABLISHED,RELATED", "-j", "ACCEPT"]).send().await?;

    let client: testcontainers::ContainerAsync<_> = node_image
        .to_owned()
        .with_exposed_port(testcontainers::core::ContainerPort::Udp(udp_port))
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(tcp_port))
        .with_privileged(true)
        .with_container_name(format!("client.{}", nanoid::nanoid!()))
        .with_cmd(["./client", "--dial", &format!("{}", &bootstrap_mu)])
        .start()
        .await?;

    let client: e2e::container::Container<_> = e2e::container::Container::new(&docker, client);

    client.connect_to(&client_lan).await?;
    client.set_default_gateway(&client_router_lan_ip).await?;

    let client_ip: std::net::Ipv4Addr = client.ip(&client_lan).await?.ok_or(anyhow::anyhow!("no connection"))?;

    let client_grpc_host: url::Host = client.host().await?;
    let client_grpc_port: u16 = client.host_ipv4_port(tcp_port).await?;
    let client_grpc_endpoint: String = format!("http://{}:{}", client_grpc_host, client_grpc_port);

    let mut client_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(client_grpc_endpoint).await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let client_peer_id_request: proto::PeerIdRequest = proto::PeerIdRequest {

    };

    let client_peer_id_response: tonic::Response<_> = client_grpc.peer_id(client_peer_id_request).await?;
    let client_peer_id_response: proto::PeerIdResponse = client_peer_id_response.into_inner();
    let client_peer_id: String = client_peer_id_response.peer_id;    

    dbg!(&client_ip);
    dbg!(&client_grpc_host);
    dbg!(&client_grpc_port);
    dbg!(&client_peer_id);

    for _ in 0..=6 {
        let client_dial_request: proto::DialRequest = proto::DialRequest {
            addr: server_mu_via_relay.to_string()
        };

        let client_dial_response: tonic::Response<_> = client_grpc.dial(client_dial_request).await?;
        let client_dial_response: proto::DialResponse = client_dial_response.into_inner();

        assert!(client_dial_response.success);

        dbg!(&client_dial_response);

        tokio::time::sleep(std::time::Duration::from_mins(1)).await;
    }

    assert!(bootstrap.can_reach(&relay_ip).await);
    assert!(bootstrap.can_reach(&client_router_wan_ip).await);
    assert!(relay.can_reach(&bootstrap_ip).await);
    assert!(relay.can_reach(&client_router_wan_ip).await);
    assert!(server_router.can_reach(&bootstrap_ip).await);
    assert!(server_router.can_reach(&relay_ip).await);
    assert!(server.can_reach(&server_router_lan_ip).await);
    assert!(server.can_reach(&bootstrap_ip).await);
    assert!(server.can_reach(&relay_ip).await); 
    assert!(client_router.can_reach(&bootstrap_ip).await);
    assert!(client_router.can_reach(&relay_ip).await);
    assert!(client.can_reach(&client_router_lan_ip).await);
    assert!(client.can_reach(&bootstrap_ip).await);
    assert!(client.can_reach(&relay_ip).await);   

    if log_dir.exists() {
        tokio::fs::remove_dir_all(&log_dir).await?;
    }

    tokio::fs::create_dir_all(&log_dir).await?;

    let containers: Vec<_> = vec![
        bootstrap,
        relay,
        client_router,
        client,
        server_router,
        server
    ];
    
    for container in containers {
        let container_id: &str = container.id();
        let path: std::path::PathBuf = log_dir.join(format!("{}.log", container_id));
        container.write_logs_to_file(&path).await?;
        container.release().await?;
    }

    wan.release().await?;
    client_lan.release().await?;
    server_lan.release().await?;

    Ok(())
}