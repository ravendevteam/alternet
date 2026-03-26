use testcontainers::ImageExt as _;
use testcontainers::runners::AsyncRunner as _;

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults()?;
    
    let lan_key: u128 = rand::random::<u128>();
    let lan = e2e::network::Network::new(&docker, format!("lan.{}", lan_key)).await?;
    
    let wan_key: u128 = rand::random::<u128>();
    let wan = e2e::network::Network::new(&docker, format!("wan.{}", wan_key)).await?;
    
    let client_router_key: u128 = rand::random::<u128>();
    let client_router: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("alpine", "latest")
        .with_container_name(format!("router.{}", client_router_key))
        .with_startup_timeout(std::time::Duration::from_mins(1))
        .with_privileged(true)
        .with_cmd(["sleep", "infinity"])
        .start()
        .await?;
    
    let client_router: e2e::container::Container<_> = e2e::container::Container::new(&docker, client_router);
    
    client_router.connect_to(&lan).await?;
    client_router.connect_to(&wan).await?;

    let client_router_lan_ip: std::net::Ipv4Addr = client_router.ip(&lan).await?.ok_or(anyhow::anyhow!("no connection"))?;
    let client_router_wan_ip: std::net::Ipv4Addr = client_router.ip(&wan).await?.ok_or(anyhow::anyhow!("no connection"))?;

    let client_router_lan_eth: usize = client_router.eth(&lan).await?;
    let client_router_lan_eth: String = format!("eth{}", client_router_lan_eth);
    let client_router_wan_eth: usize = client_router.eth(&wan).await?;
    let client_router_wan_eth: String = format!("eth{}", client_router_wan_eth);

    client_router.exec().args(vec!["apk", "add", "iptables"]).send().await?;
    client_router.exec().args(vec!["sysctl", "-w", "net.ipv4.ip_forward=1"]).send().await?;
    client_router.exec().args(vec!["iptables", "-t", "nat", "-A", "POSTROUTING", "-o", &client_router_wan_eth, "-j", "MASQUERADE"]).send().await?;
    client_router.exec().args(vec!["iptables", "-A", "FORWARD", "-i", &client_router_lan_eth, "-o", &client_router_wan_eth, "-j", "ACCEPT"]).send().await?;    
    client_router.exec().args(vec!["iptables", "-A", "FORWARD", "-m", "conntrack", "--ctstate", "ESTABLISHED,RELATED", "-j", "ACCEPT"]).send().await?;

    let client_key: u128 = rand::random::<u128>();
    let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("alpine", "latest")
        .with_container_name(format!("client.{}", client_key))
        .with_startup_timeout(std::time::Duration::from_mins(1))
        .with_privileged(true)
        .with_cmd(["sleep", "infinity"])
        .start()
        .await?;

    let client: e2e::container::Container<_> = e2e::container::Container::new(&docker, client);

    client.exec().args(vec!["apk", "add", "mtr"]).send().await?;
    client.connect_to(&lan).await?;
    client.set_default_gateway(&client_router_lan_ip).await?;

    let client_ip: std::net::Ipv4Addr = client.ip(&lan).await?.ok_or(anyhow::anyhow!("no connection"))?;

    let server_key: u128 = rand::random::<u128>();
    let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("alpine", "latest")
        .with_container_name(format!("server.{}", server_key))
        .with_startup_timeout(std::time::Duration::from_mins(1))
        .with_privileged(true)
        .with_cmd(["sleep", "infinity"])
        .start()
        .await?;

    let server: e2e::container::Container<_> = e2e::container::Container::new(&docker, server);

    server.connect_to(&wan).await?;
    server.open_tcp_port(8080).await?;
    server.open_udp_port(8080).await?;

    let server_ip = server.ip(&wan).await?.ok_or(anyhow::anyhow!("no connection"))?;

    assert!(client_router.can_reach(&client_ip).await);
    assert!(client_router.can_reach(&server_ip).await);

    assert!(client.can_reach(&client_router_lan_ip).await);
    assert!(server.can_reach(&client_router_wan_ip).await);

    assert!(client.can_reach(&server_ip).await);

    let (stdout, _) = client.exec()
        .arg("mtr")
        .arg("--report")
        .arg("--report-cycles")
        .arg("1")
        .arg(format!("{}", server_ip))
        .send()
        .await?;

    dbg!(client_router_lan_ip);
    dbg!(client_router_wan_ip);
    dbg!(client_ip);
    dbg!(server_ip);

    println!("{}", stdout);

    assert!(stdout.contains(&format!("{}", server_ip)));

    client_router.release().await?;
    client.release().await?;
    server.release().await?;

    lan.release().await?;
    wan.release().await?;
    Ok(())
}