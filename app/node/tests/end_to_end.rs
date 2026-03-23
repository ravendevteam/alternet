#![allow(clippy::vec_init_then_push)]
#![allow(unused)]
#![cfg(feature = "end_to_end")]

use std::io::Read as _;
use std::num::NonZeroI128;
use std::os::unix::process::CommandExt;
use std::thread::spawn;
use futures_util::StreamExt as _;
use futures_util::TryStreamExt as _;
use testcontainers::Image;
use testcontainers::ImageExt;
use testcontainers::ImageExt as _;
use testcontainers::runners::AsyncRunner as _;
use tokio::io::AsyncWriteExt;

mod net {
    pub type Docker = bollard::Docker;
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[derive(bon::Builder)]
    pub struct Configuration {
        #[builder(into)]
        name: String,
        #[builder(into)]
        driver: Option<String>,
        #[builder(into)]
        scope: Option<String>,
        #[builder(into)]
        ingress: Option<bool>,
        #[builder(into)]
        internal: Option<bool>,
        #[builder(into)]
        attachable: Option<bool>,
        #[builder(into)]
        config_only: Option<bool>,
        #[builder(into)]
        config_from: Option<bollard::secret::ConfigReference>,
        #[builder(into)]
        ipam: Option<bollard::secret::Ipam>,
        #[builder(into)]
        enable_ipv4: Option<bool>,
        #[builder(into)]
        enable_ipv6: Option<bool>,
        #[builder(into)]
        options: Option<std::collections::HashMap<String, String>>,
        #[builder(into)]
        labels: Option<std::collections::HashMap<String, String>>
    }

    impl Into<bollard::secret::NetworkCreateRequest> for Configuration {
        fn into(self) -> bollard::secret::NetworkCreateRequest {
            let Self {
                name,
                driver,
                scope,
                ingress,
                internal,
                attachable,
                config_only,
                config_from,
                ipam,
                enable_ipv4,
                enable_ipv6,
                options,
                labels
            } = self;
            bollard::secret::NetworkCreateRequest {
                name,
                driver,
                scope,
                internal,
                attachable,
                ingress,
                config_only,
                config_from,
                ipam,
                enable_ipv4,
                enable_ipv6,
                options,
                labels
            }
        }
    }

    pub trait Ext {
        async fn create_network(&self, name: &str, configuration: Option<Configuration>) -> Result<()>;
        async fn remove_network(&self, name: &str) -> Result<()>;
    }

    impl Ext for Docker {
        async fn create_network(&self, name: &str, configuration: Option<Configuration>) -> Result<()> {
            <Self as Ext>::remove_network(self, name).await.ok();
            if let Some(configuration) = configuration {
                let configuration = configuration.into();
                self.create_network(configuration).await?;
            } else {
                let configuration = Configuration::builder()
                    .name(name)
                    .build();
                let configuration = configuration.into();
                self.create_network(configuration).await?;
            }
            Ok(())
        }

        async fn remove_network(&self, name: &str) -> Result<()> {
            let images: Vec<_> = self.list_containers(None).await?;
            for image in images {
                let Some(image_name) = image.id else {
                    continue
                };
                let request: bollard::secret::NetworkDisconnectRequest = bollard::secret::NetworkDisconnectRequest {
                    container: image_name,
                    force: Some(true)
                };
                self.disconnect_network(name, request).await.ok();
            }
            self.remove_network(name).await.ok();
            Ok(())
        }
    }
}

mod router {
    #![allow(async_fn_in_trait)]

    use super::*;

    use testcontainers::ImageExt;
    use testcontainers::runners::AsyncRunner as _;

    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    pub type Docker = bollard::Docker;

    #[derive(Debug)]
    #[derive(derive_more::Deref)]
    #[derive(derive_more::DerefMut)]
    pub struct Image<'a> {
        docker: &'a Docker,
        #[deref]
        #[deref_mut]
        pub x: testcontainers::ContainerAsync<testcontainers::GenericImage>
    }

    // model
    // ... > router > internet < router < ...

    impl<'a> Image<'a> {
        pub fn new(docker: &'a Docker, internal: testcontainers::ContainerAsync<testcontainers::GenericImage>) -> Image<'a> {
            Self {
                docker,
                x: internal
            }
        }

        pub async fn ip(&self, network: &str) -> Result<Option<std::net::Ipv4Addr>> {
            let name: &str = self.id();
            let response: bollard::secret::ContainerInspectResponse = self.docker.inspect_container(name, None).await?;        
            if let Some(response) = response.network_settings
            && let Some(response) = response.networks
            && let Some(response) = response.get(network)
            && let Some(response) = response.ip_address.as_ref() {
                let ret: std::net::Ipv4Addr = response.parse()?;
                return Ok(Some(ret))
            } 
            Ok(None)
        }

        pub async fn eth(&self, network: &str) -> Result<String> {
            let ip: std::net::Ipv4Addr = self.ip(network).await?.ok_or("not connected to network")?;
        
            let output: String = self.exec_wait(vec!["sh", "-c", &format!("ip -o addr show | awk '{{print $2, $4}}' | grep '^[^:]* *{}'", ip)]).await?;

            for line in output.lines() {
                let Some(start_key) = line.find("eth") else {
                    continue
                };            
                let Some(final_key) = line.find(char::is_whitespace) else {
                    continue
                };
                let ret: String = line[start_key..final_key].to_owned();
                return Ok(ret);
            }
            Err(
                format!("no interface found for network `{}` (ip={})", network, ip).into()
            )
        }

        pub async fn exec_wait(&self, cmd: Vec<&str>) -> Result<String> {
            // ffs testcontainers why make me do this : ( - not cool
            let cmd_copy_a: Vec<_> = cmd.to_owned();
            let cmd_copy_b: Vec<_> = cmd.to_owned();
            let mut outcome: testcontainers::core::ExecResult = self.exec(testcontainers::core::ExecCommand::new(cmd_copy_a)).await?;
            if !outcome.success().await? {
                let output: String = outcome.read().await;
                return Err(
                    format!("command failure: `{:?}`: {}", cmd_copy_b, output).into()
                )
            }
            let output: String = outcome.read().await;
            Ok(output)
        }

        pub async fn connect_to_network(&self, network: &str, configuration: Option<bollard::secret::EndpointSettings>) -> Result<()> {
            let configuration: bollard::secret::NetworkConnectRequest = bollard::secret::NetworkConnectRequest {
                container: self.id().to_owned(),
                endpoint_config: configuration
            };
            self.docker.connect_network(network, configuration).await?;
            Ok(())
        }

        pub async fn connect_to_wan_as_transit_router(&self) -> Result<()> {
            self.exec(testcontainers::core::ExecCommand::new(["sysctl", "-w", "net.ipv4.ip_forward=1"])).await?;
            Ok(())
        }

        pub async fn connect_to_lan_as_router(&self, lan: &str, wan: &str, wan_gateway_ip: &std::net::Ipv4Addr) -> Result<()> {
            let wan_gateway_ip_str: String = wan_gateway_ip.to_string();
            self.connect_to_network(lan, None).await?;
            self.connect_to_network(wan, None).await?;
            let lan_eth: String = self.eth(lan).await?;
            let wan_eth: String = self.eth(wan).await?;
            self.exec_wait(vec!["sysctl", "-w", "net.ipv4.ip_forward=1"]).await?;
            self.exec_wait(vec!["iptables", "-P", "FORWARD", "DROP"]).await?;
            self.exec_wait(vec!["iptables", "-t", "nat", "-A", "POSTROUTING", "-o", &wan_eth, "-j", "MASQUERADE"]).await?;
            self.exec_wait(vec!["iptables", "-A", "FORWARD", "-i", &lan_eth, "-o", &wan_eth, "-j", "ACCEPT"]).await?;
            self.exec_wait(vec!["iptables", "-A", "FORWARD", "-i", &wan_eth, "-o", &lan_eth, "-m", "conntrack", "--ctstate", "ESTABLISHED,RELATED", "-j", "ACCEPT"]).await?;
            self.exec_wait(vec!["ip", "route", "del", "default", "2>/dev/null", "||", "true"]).await?;
            self.exec_wait(vec!["ip", "route", "add", "default", "via", &wan_gateway_ip_str]).await?;
            Ok(())
        }

        pub async fn connect_to_lan_router<'b>(&self, lan_router: &Image<'b>, lan: &str) -> Result<()> {
            let lan_router_ip: std::net::Ipv4Addr = lan_router.ip(lan).await?.ok_or("ip unavailable")?;
            let lan_router_ip_str: String = lan_router_ip.to_string();
            self.connect_to_network(lan, None).await?;
            self.exec_wait(vec!["ip", "route", "del", "default", "2>/dev/null", "||", "true"]).await?;
            self.exec_wait(vec!["ip", "route", "add", "default", "via", &lan_router_ip_str]).await?;
            Ok(())
        }

        pub async fn logs(&self) -> Vec<String> {
            use tokio::io::AsyncReadExt as _;
            let mut stdout_bytes = Vec::new();
            let mut stderr_bytes = Vec::new();
            let _ = self.stdout(false).read_to_end(&mut stdout_bytes).await;
            let _ = self.stderr(false).read_to_end(&mut stderr_bytes).await;
            let stdout_str = String::from_utf8_lossy(&stdout_bytes);
            let stderr_str = String::from_utf8_lossy(&stderr_bytes);
            stdout_str
                .lines()
                .chain(
                    stderr_str.lines()
                )
                .map(String::from)
                .collect()
        }
    }



    #[derive(derive_more::Deref)]
    #[derive(derive_more::DerefMut)]
    pub struct Router<'a> {
        #[deref]
        #[deref_mut]
        pub image: Image<'a>,
        lan: String,
        wan: String
    }

    impl<'a> Router<'a> {
        pub async fn new(image: Image<'a>, lan: String, wan: String) -> Result<Self> {
            image.exec_wait(vec!["apk", "add", "--no-cache", "iptables", "iproute2"]).await?;
            let wan_gateway_ip: bollard::secret::NetworkInspect = image.docker.inspect_network(&wan, None).await?;
            let wan_gateway_ip: bollard::secret::Ipam = wan_gateway_ip.ipam.ok_or("")?;
            let wan_gateway_ip: &bollard::secret::IpamConfig = wan_gateway_ip.config.as_ref().ok_or("")?.first().ok_or("")?;
            let wan_gateway_ip: &str = wan_gateway_ip.gateway.as_ref().ok_or("")?;
            let wan_gateway_ip: std::net::Ipv4Addr = wan_gateway_ip.parse()?;
            image.connect_to_network(&wan, None).await?;
            image.connect_to_network(&lan, None).await?;
            let wan_eth: String = image.eth(&wan).await?;
            let lan_eth: String = image.eth(&lan).await?;
            image.exec_wait(vec!["sh", "-c", "sysctl -w net.ipv4.ip_forward=1"]).await?;
            image.exec_wait(vec!["sh", "-c", &format!("iptables -P FORWARD DROP")]).await?;
            image.exec_wait(vec!["sh", "-c", &format!("iptables -t nat -A POSTROUTING -o {} -j MASQUERADE", wan_eth)]).await?;
            image.exec_wait(vec!["sh", "-c", &format!("iptables -A FORWARD -i {} -o {} -j ACCEPT", lan_eth, wan_eth)]).await?;
            image.exec_wait(vec!["sh", "-c", &format!("iptables -A FORWARD -i {} -o {} -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT", wan_eth, lan_eth)]).await?;
            image.exec_wait(vec!["sh", "-c", &format!("ip route del default")]).await?;
            image.exec_wait(vec!["sh", "-c", &format!("ip route add default via {}", wan_gateway_ip)]).await?;
            let new: Self = Self {
                image,
                lan,
                wan
            };
            Ok(new)
        }

        pub async fn register(&self, client: &Image<'a>) -> Result<()> {
            let lan_ip: std::net::Ipv4Addr = self.image.ip(&self.lan).await?.ok_or("router not connected to its own lan")?;
            let lan_ip_str: String = lan_ip.to_string();
            client.connect_to_network(&self.lan, None).await?;
            client.exec_wait(vec!["ip", "route", "del", "default"]).await?;
            client.exec_wait(vec!["ip", "route", "add", "default", "via", &lan_ip_str]).await?;
            Ok(())
        }

        pub async fn register_public(&self, public: &Image<'a>) -> Result<()> {
            let wan_ip = self.ip(&self.wan).await?.ok_or("router not connected to wan")?;
            let wan_ip_str = wan_ip.to_string();

            public.connect_to_network(&self.wan, None).await?;

            let lan = self.docker.inspect_network(&self.lan, None).await?;

            let wan_ip = lan.ipam.ok_or("missing ipam")?;
            let wan_ip = wan_ip.config.as_ref().ok_or("missing config")?;
            let wan_ip = wan_ip.first().ok_or("missing subnet")?;
            let subnet = wan_ip.subnet.as_ref().ok_or("missing subnet")?;

            public.exec_wait(vec!["ip", "route", "add", subnet, "via", &wan_ip_str]).await?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[tokio::test]
        async fn router_firewall_and_nat() -> Result<()> {
            let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults()?;
            let wan: &str = "wan";
            let lan: &str = "lan";

            net::Ext::create_network(&docker, wan, None).await?;
            net::Ext::create_network(&docker, lan, None).await?;

            let router: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("alpine", "latest")
                .with_cmd(["sleep", "infinity"])
                .with_privileged(true)
                .start()
                .await?;

            let router: Image = Image::new(&docker, router);
            let router: Router = Router::new(router, lan.to_owned(), wan.to_owned()).await?;

            let router_lan_ip = router.ip(lan).await?.ok_or("router not connected to lan")?;
            let router_lan_ip_str = router_lan_ip.to_string();

            let router_wan_ip: std::net::Ipv4Addr = router.ip(wan).await?.ok_or("router ip unavailable")?;
            let router_wan_ip_str: String = router_wan_ip.to_string();

            let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("alpine", "latest")
                .with_privileged(true)
                .with_cmd(["sleep", "infinity"])
                .start()
                .await?;
            
            let client: Image = Image::new(&docker, client);

            router.register(&client).await?;

            let client_lan_ip: std::net::Ipv4Addr = client.ip(lan).await?.ok_or("client not connected")?;
            let client_lan_ip_str: String = client_lan_ip.to_string();

            let public_peer: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("alpine", "latest")
                .with_privileged(true)
                .with_cmd(["sleep", "infinity"])
                .start()
                .await?;

            let public_peer: Image = Image::new(&docker, public_peer);

            router.register_public(&public_peer).await?;

            let public_peer_wan_ip: std::net::Ipv4Addr = public_peer.ip(wan).await?.ok_or("public peer not connected")?;
            let public_peer_wan_ip_str: String = public_peer_wan_ip.to_string();

            client.exec_wait(vec!["ping", "-c", "1", "-W", "1", &router_lan_ip_str]).await.expect("client to ping router");
            public_peer.exec_wait(vec!["ping", "-c", "1", "-W", "3", &router_wan_ip_str]).await.expect("public peer to ping router");
    
            super::net::Ext::remove_network(&docker, wan).await?;
            net::Ext::remove_network(&docker, lan).await?;
            Ok(())
        }
    }


    trait S {
        async fn success(&mut self) -> Result<bool>;
        async fn lines(&mut self) -> Vec<String>;
        async fn read(&mut self) -> String;
    }

    impl S for testcontainers::core::ExecResult {
        async fn success(&mut self) -> Result<bool> {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let Some(code) = self.exit_code().await? else {
                    continue
                };
                return Ok(code == 0)    
            }
        }

        async fn lines(&mut self) -> Vec<String> {
            let stdout_bytes: Vec<_> = self.stdout_to_vec().await.unwrap_or_default();
            let stdout_str: std::borrow::Cow<_> = String::from_utf8_lossy(&stdout_bytes);
            let stderr_bytes: Vec<_> = self.stderr_to_vec().await.unwrap_or_default();
            let stderr_str: std::borrow::Cow<_> = String::from_utf8_lossy(&stderr_bytes);
            let stderr_lines: std::str::Lines = stderr_str.lines();
            stdout_str.lines().chain(stderr_lines).map(String::from).collect()
        }

        async fn read(&mut self) -> String {
            self.lines().await.join("\n")
        }
    }
}

mod proto {
    include!("../proto_target/an.rs");
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod log {
    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug)]
    #[derive(strum::EnumCount)]
    #[derive(thiserror::Error)]
    pub enum Error {
        #[error("{0}")]
        Io(#[from] std::io::Error),
        #[error("unable to parse")]
        #[strum(serialize = "unparsable")]
        #[strum(serialize = "Unparsable")]
        #[strum(serialize = "UNPARSABLE")]
        Unparsable,
        #[error("not a log file")]
        #[strum(serialize = "not_log_file")]
        NotLogFile,
        #[error("")]
        FileNotFound
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(Copy)]
    #[derive(PartialEq)]
    #[derive(Eq)]
    #[derive(Default)]
    #[derive(strum::EnumCount)]
    #[derive(strum::EnumIter)]
    #[derive(strum::EnumString)]
    pub enum Severity {
        #[default]
        #[strum(serialize = "INFO")]
        Info,
        #[strum(serialize = "WARN")]
        Warn,
        #[strum(serialize = "ERROR")]
        Error,
        #[strum(serialize = "DEBUG")]
        Debug
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(PartialEq)]
    #[derive(Eq)]
    #[derive(getset::Getters)]
    #[derive(getset::CopyGetters)]
    #[derive(bon::Builder)]
    pub struct Log {
        #[getset(get_copy = "pub")]
        #[builder(into)]
        #[builder(default = chrono::Utc::now())]
        timestamp: chrono::DateTime<chrono::Utc>,
        #[getset(get = "pub")]
        #[builder(into)]
        #[builder(default = Severity::Info)]
        severity: Severity,
        #[getset(get = "pub")]
        #[builder(into)]
        component: String,
        #[getset(get = "pub")]
        #[builder(into)]
        message: String
    }

    impl std::str::FromStr for Log {
        type Err = Error;

        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            let (timestamp, more) = s.split_once(' ').ok_or(Error::Unparsable)?;
            let timestamp: chrono::DateTime<chrono::Utc> = timestamp.parse().ok().ok_or(Error::Unparsable)?;
            let lb: usize = more.find('[').ok_or(Error::Unparsable)?;
            let rb: usize = more.find(']').ok_or(Error::Unparsable)?;
            let meta: &str = &more[lb + 1..rb];
            let (severity, component) = meta.split_once(' ').ok_or(Error::Unparsable)?;
            let message: &str = more[rb + 1..].trim();
            let message: String = message.to_owned();
            let severity: Severity = severity
                .parse()
                .ok()
                .ok_or(Error::Unparsable)?;
            let component: String = component.to_owned();
            let new: Self = Self {
                timestamp,
                severity,
                component,
                message
            };
            Ok(new)
        }
    }

    #[derive(Debug)]
    #[derive(Clone)]
    #[derive(PartialEq)]
    #[derive(Eq)]
    pub struct Report {
        id_to_logs: std::collections::HashMap<String, Vec<Log>>
    }

    impl Report {
        pub fn from_log_file_paths(paths: &[std::path::PathBuf]) -> Result<Self> {
            let mut id_to_logs: std::collections::HashMap<String, Vec<Log>> = std::collections::HashMap::new();
            for path in paths {
                let file_name: std::ffi::OsString = path.file_name().ok_or(Error::FileNotFound)?.to_owned();
                let file_name: String = file_name
                    .to_string_lossy()
                    .to_string();
                let (file_name, extension) = file_name.split_once('.').ok_or(Error::Unparsable)?;
                if extension != "log" {
                    return Err(Error::NotLogFile)
                }
                let file_name: String = file_name.to_owned();
                let content: String = std::fs::read_to_string(path)?;
                let mut logs: Vec<_> = vec![];
                for line in content.lines() {
                    let log: Log = line.parse()?;
                    logs.push(log);
                }
                id_to_logs.insert(file_name, logs);
            }
            let new: Self = Self {
                id_to_logs
            };
            Ok(new)
        }

        pub fn from_dir(dir: &std::path::Path) -> Result<Self> {
            let mut paths = vec![];
            for item in std::fs::read_dir(dir)? {
                let item: std::fs::DirEntry = item?;
                let item_path: std::path::PathBuf = item.path();
                let item_file_type: std::fs::FileType = item.file_type()?;
                if item_file_type.is_file() {
                    paths.push(item_path);
                }
            }
            Self::from_log_file_paths(&paths)
        }
    }

    impl Report {
        /// # Proof of Startup
        /// Every node starts successfully.
        pub fn is_proof_of_startup(&self) -> bool {
            self.id_to_logs.values().all(|logs| {
                let mut has_boot: bool = false;
                let mut has_id: bool = false;
                let mut has_looped: bool = false;
                for log in logs {
                    let msg: &str = log.message();
                    if msg.contains("booting") {
                        has_boot = true;
                    }
                    if msg.contains("peer identity initialized") {
                        has_id = true;
                    }
                    if msg.contains("entering event loop") {
                        has_looped = true;
                    }
                }
                has_boot && has_id && has_looped
            })
        }

        pub fn is_proof_of_relay_usage(&self) -> bool {
            self.id_to_logs.values().any(|logs| {
                logs.iter().any(|log| {
                    let msg: String = log.message().to_lowercase();
                    let has_relay: bool = msg.contains("relay");
                    let has_dial: bool = msg.contains("dial");
                    let has_forward: bool = msg.contains("forward");
                    let has_circuit: bool = msg.contains("circuit");
                    has_relay && (has_dial || has_forward || has_circuit)
                })
            })
        }

        /// # Proof of GRPC Interaction
        pub fn is_proof_of_grpc_interaction(&self) -> bool {
            self.id_to_logs.values().any(|logs| {
                logs.iter().any(|log| {
                    log.component().contains("grpc") && log.message().contains("dial request")
                })
            })
        }

        pub fn is_proof_of_routing_table_population(&self) -> bool {
            self.id_to_logs.values().all(|logs| {
                logs.iter().any(|log| {
                    log.message().contains("knows about") && log.message().contains("peers")
                })
            })
        }

        pub fn is_proof_of_unique_identity(&self) -> bool {
            let mut seen: std::collections::HashMap<&str, bool> = std::collections::HashMap::new();
            for (_, logs) in self.id_to_logs.iter() {
                for log in logs {
                    let msg: &str = log.message();
                    if msg.contains("PeerId")
                    && let Some(from) = msg.find('(')
                    && let Some(to) = msg.find(')') {
                        let peer_id: &str = &msg[from..to];
                        if seen.insert(peer_id, true).is_some() {
                            return false
                        }
                    }
                }
            }
            true
        }

        pub fn is_proof_of_sybil_attack(&self) -> bool {
            // placeholder
            self.id_to_logs.values().any(|logs| {
                logs.iter().any(|log| {
                    let msg = log.message().to_lowercase();
                    msg.contains("sybil")
                    || msg.contains("malicious")
                    || msg.contains("routing pollution")
                    || msg.contains("invalid peer")
                })
            })
        }

        pub fn is_proof_of_eventual_discovery(&self) -> bool {
            self.id_to_logs.values().all(|logs| {
                logs.iter().any(|line| {
                    line.message().contains("knows about") && !line.message().contains("knows no one")
                })
            })
        }
    
        pub fn is_proof_of_successful_bootstrap(&self) -> bool {
            self.id_to_logs.values().any(|logs| {
                logs.iter().any(|line| {
                    line.message().contains("bootstrap complete, remaining: 0")
                })
            })
        }

        pub fn is_proof_of_causality(
            &self,
            lhs_container_id: &str,
            lhs_log_message: &str,
            rhs_container_id: &str,
            rhs_log_message: &str
        ) -> bool {
            let lhs_time = self.id_to_logs[lhs_container_id].iter()
                .find(|line| {
                    line.message.contains(lhs_log_message)
                })
                .map(|line| {
                    line.timestamp
                });
            let Some(lhs_time) = lhs_time else {
                return false
            };
            let rhs_time = self.id_to_logs[rhs_container_id].iter()
                .find(|line| {
                    line.message.contains(rhs_log_message)
                })
                .map(|line| {
                    line.timestamp
                });
            let Some(rhs_time) = rhs_time else {
                return false
            };
            lhs_time < rhs_time
        }

        pub fn is_proof_of_convergence(&self, expected_peer_count: usize) -> bool {
            for (_, logs) in self.id_to_logs.iter() {
                let reached: bool = logs.iter().any(|line| {
                    line.message.contains("knows about") && line.message.contains(&format!("{} peers", expected_peer_count))
                });
                if !reached {
                    return false
                }
            }
            true
        }

        /// # Proof of Connectivity Persistence
        /// Once the expected peer count is reached, it never drops again.
        pub fn is_proof_of_connectivity_persistence(&self, expected_count: usize) -> bool {
            for logs in self.id_to_logs.values() {
                let mut converged: bool = false;
                for line in logs {
                    if line.message().contains("knows about") && line.message.contains(&format!("{} peers", expected_count)) {
                        converged = true;
                    }
                    if converged && line.message.contains("peer count dropped") {
                        return false
                    }
                }
            }
            true
        }

        /// # Proof of Cohesion
        /// There are no partitioned nodes and the network's topology is connected.
        pub fn is_proof_of_cohesion(&self) -> bool {
            use std::collections::{HashMap, HashSet, VecDeque};
            let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

            for logs in self.id_to_logs.values() {
                let mut peer_id: Option<String> = None;

                for log in logs {
                    if !log.message().contains("PeerId") {
                        continue;
                    }

                    let msg = log.message();
                    let Some(start) = msg.find('(') else { continue };
                    let Some(end) = msg.find(')') else { continue };

                    peer_id = Some(msg[start + 1..end].to_string());
                }

                let Some(peer_id) = peer_id else { continue };

                let Some(log) = logs.iter().rfind(|log| log.message().contains("knows about")) else {
                    continue;
                };

                let msg = log.message();

                let Some(start) = msg.find('[') else { continue };
                let Some(end) = msg.find(']') else { continue };

                let peer_ids: Vec<String> = msg[start + 1..end]
                    .split(',')
                    .map(|id| id.trim().to_string())
                    .collect();

                for p in &peer_ids {
                    graph.entry(peer_id.clone()).or_default().insert(p.clone());
                    graph.entry(p.clone()).or_default().insert(peer_id.clone());
                }
            }

            if graph.is_empty() {
                return false;
            }

            // BFS to check connectivity
            let start = graph.keys().next().unwrap().clone();
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();

            queue.push_back(start.clone());
            visited.insert(start);

            while let Some(node) = queue.pop_front() {
                if let Some(neighbors) = graph.get(&node) {
                    for n in neighbors {
                        if visited.insert(n.clone()) {
                            queue.push_back(n.clone());
                        }
                    }
                }
            }

            visited.len() == graph.len()
        }

        /// # Proof of Stability
        /// Churn decreases over time or is below the threshold after the initial discovery phase.
        pub fn is_proof_of_stability(&self, churn_threshold: usize) -> bool {
            for (_, logs) in self.id_to_logs.iter() {
                let churn_vals: Vec<_> = logs
                    .iter()
                    .filter(|log| {
                        log.message().contains("routing churn:")
                    })
                    .filter_map(|log| {
                        log.message()
                            .split("routing churn: ")
                            .nth(1)?
                            .parse::<usize>()
                            .ok()
                    })
                    .collect();
                if let Some(&last_churn) = churn_vals.last() && last_churn > churn_threshold {
                    return false
                }
            }
            true
        }

        pub fn is_proof_of_propagation(&self, origin_id: &str, msg: &str) -> bool {
            let sent: bool = self.id_to_logs
                .get(origin_id)
                .map(|logs| {
                    logs.iter().any(|log| {
                        log.message().contains("broadcast") && log.message().contains(msg)
                    })
                })
                .unwrap_or(false);
            if !sent {
                return false
            }
            self.id_to_logs
                .iter()
                .filter(|(id, _)| {
                    *id != origin_id
                })
                .all(|(_, logs)| {
                    logs.iter().any(|log| {
                        log.message().contains("received") && log.message().contains(msg)
                    })
                })
        }
    }
}

mod network {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    pub const A: &str = "an_a";
    pub const B: &str = "an_b";
    pub const C: &str = "an_c";

    #[derive(getset::Getters)]
    pub struct Network<'a> {
        docker: &'a bollard::Docker,
        #[getset(get = "pub")]
        name: String
    }

    #[bon::bon]
    impl<'a> Network<'a> {
        #[builder]
        #[builder(finish_fn = "reserve")]
        pub async fn new(
            docker: &'a bollard::Docker,
            docker_network_conf: bollard::secret::NetworkCreateRequest,
            #[builder(into)]
            name: String
        ) -> Result<Self> {
            docker.create_network(docker_network_conf).await?;
            let new: Self = Self {
                docker,
                name
            };
            Ok(new)
        }
    }

    impl<'a> Network<'a> {
        pub async fn release(self) {
            let containers: Vec<_> = self.docker.list_containers(None).await.unwrap_or_default();
            for container in containers {
                let Some(id) = container.id else {
                    continue
                };
                let request: bollard::secret::NetworkDisconnectRequest = bollard::secret::NetworkDisconnectRequest {
                    container: id,
                    force: Some(true)
                };
                self.docker.disconnect_network(&self.name, request).await.ok();
            }
            self.docker.remove_network(&self.name).await.ok();
        }
    }
}

type Container = testcontainers::ContainerAsync<testcontainers::GenericImage>;

trait Docker {
    async fn load(&self, path: &std::path::Path) -> Result<()>;
    async fn load_built_tar_image_from_ws_target_dir(&self) -> Result<()>;
    async fn reset(&self) -> Result<()>;
    async fn reset_network(&self, network_name: &str) -> Result<()>;
    async fn write_logs_to_file(&self, out_dir: &std::path::Path, containers: Vec<Container>) -> Result<()>;
}

impl Docker for bollard::Docker {
    async fn load(&self, path: &std::path::Path) -> Result<()> {
        let mut file = std::fs::File::open(path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();
        let options = bollard::query_parameters::ImportImageOptions {
            quiet: false,
            ..Default::default()
        };
        let mut stream = self.import_image(options, bollard::body_full(bytes.into()), None);
        while let Some(_) = stream.try_next().await.unwrap() {

        }
        Ok(())
    }

    async fn load_built_tar_image_from_ws_target_dir(&self) -> Result<()> {
        std::process::Command::new("cargo")
            .arg("run")
            .arg("--package")
            .arg("task")
            .arg("build-image")
            .spawn()
            .expect("failed to build image")
            .wait()
            .expect("failed to build image");
        
        let ws_dir: std::path::PathBuf = cargo_metadata::MetadataCommand::new()
            .exec()
            .unwrap()
            .workspace_root
            .to_string()
            .into();
        
        let image_dir: std::path::PathBuf = ws_dir
            .join("target")
            .join("image");
        
        let image_path: std::path::PathBuf = image_dir.join("node.tar");
        
        self.load(&image_path).await;
        Ok(())
    }

    async fn reset(&self) -> Result<()> {
        let containers: Vec<_> = self.list_containers(None).await?;
        for container in containers {
            self.stop_container(&container.id.to_owned().unwrap(), None).await?;
        }
        Ok(())
    }

    async fn reset_network(&self, network_name: &str) -> Result<()> {
        let containers: Vec<_> = self.list_containers(None).await?;
        for container in containers {
            let Some(id) = container.id else {
                continue
            };
            let request: bollard::secret::NetworkDisconnectRequest = bollard::secret::NetworkDisconnectRequest {
                container: id,
                force: Some(true)
            };
            self.disconnect_network(network_name, request).await?;
        }
        Ok(())   
    }

    async fn write_logs_to_file(&self, out_dir: &std::path::Path, containers: Vec<testcontainers::ContainerAsync<testcontainers::GenericImage>>) -> Result<()> {
        std::fs::remove_dir_all(out_dir).ok();
        std::fs::create_dir_all(out_dir)?;
        let logs_conf: bollard::query_parameters::LogsOptions = bollard::query_parameters::LogsOptions {
            stdout: true,
            stderr: true,
            timestamps: true,
            tail: "all".into(),
            ..Default::default()
        };
        for container in containers {
            let logs_conf: bollard::query_parameters::LogsOptions = logs_conf.to_owned();
            let container_id: &str = container.id();
            let mut container_path: std::path::PathBuf = out_dir.join(container_id);
            container_path.set_extension("log");
            let mut file: tokio::fs::File = tokio::fs::File::create(container_path).await.unwrap();
            let mut stream = self.logs(container_id, Some(logs_conf));
            while let Some(log) = stream.next().await {
                let log: bollard::container::LogOutput = log?;
                let bytes = match log {
                    bollard::container::LogOutput::StdOut {
                        message
                    } => {
                        message
                    },
                    bollard::container::LogOutput::StdErr {
                        message
                    } => {
                        message
                    },
                    bollard::container::LogOutput::Console {
                        message
                    } => {
                        message
                    },
                    _ => continue
                };
                file.write_all(&bytes).await.unwrap()
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
trait Test {
    async fn run(&self, docker: &bollard::Docker);
}

struct Harness {
    docker: bollard::Docker,
    tests: Vec<Box<dyn Test>>
}

impl Harness {
    pub fn new(docker: bollard::Docker) -> Self {
        let tests: Vec<_> = vec![];
        Self {
            docker,
            tests
        }
    }
}

impl Harness {
    pub fn add_test<T>(&mut self, test: T)
    where
        T: Test,
        T: 'static {
        self.tests.push(Box::new(test));
    }

    pub async fn launch(self) {
        self.docker.reset().await.ok();
        // self.docker.load_built_tar_image_from_ws_target_dir().await.unwrap();
        for test in self.tests.iter() {
            test.run(&self.docker).await;
            self.docker.reset().await.ok();
        }
    }
}

struct NatEasy;

#[async_trait::async_trait]
impl Test for NatEasy {
    async fn run(&self, docker: &bollard::Docker) {
        let log_dir: std::path::PathBuf = std::path::PathBuf::new()
            .join("tests")
            .join("log")
            .join("nat_easy");

        let network_a_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::A.to_owned(),
            driver: None,
            scope: None,
            internal: None,
            attachable: None,
            ingress: None,
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: None,
            enable_ipv6: None,
            options: None,
            labels: None
        };

        let network_b_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::B.to_owned(),
            driver: None,
            scope: None,
            internal: None,
            attachable: None,
            ingress: None,
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: None,
            enable_ipv6: None,
            options: None,
            labels: None
        };

        let network_a: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_a_conf)
            .name(network::A)
            .reserve()
            .await
            .unwrap();
    
        let network_b: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_b_conf)
            .name(network::B)
            .reserve()
            .await
            .unwrap();

        let udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let tcp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

        let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./bootstrap"])
            .with_network(network_a.name())
            .with_network(network_b.name())
            .start()
            .await
            .unwrap();

        let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.expect("bridge ip addr");
        let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

        let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./relay", "--dial", &bootstrap_addr])
            .with_network(network_a.name())
            .with_network(network_b.name())
            .start()
            .await
            .expect("successful container launch");

        let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./server", "--dial", &bootstrap_addr])
            .with_network(network_b.name())
            .start()
            .await
            .expect("successful container launch");

        let server_ip: std::net::IpAddr = server.get_bridge_ip_address().await.expect("bridge ip addr");
        let server_addr: String = format!("/ip4/{}/udp/4001/quic-v1", server_ip);

        let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./client", "--dial", &bootstrap_addr])
            .with_network(network_a.name())
            .start()
            .await
            .expect("successful container launch");

        let client_grpc_port: u16 = client.get_host_port_ipv4(8080).await.expect("host port ipv4");
        let client_gprc_endpoint: String = format!("http://127.0.0.1:{}", client_grpc_port);
        let mut client_grpc: proto::node_client::NodeClient<_> = wait_for_grpc(client_gprc_endpoint).await;

        let client_request: proto::DialRequest = proto::DialRequest {
            addr: server_addr    
        };

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        client_grpc.dial(client_request).await.expect("successful dial");
        
        tokio::time::sleep(std::time::Duration::from_mins(1)).await;

        let mut logged: Vec<_> = vec![];
        logged.push(bootstrap);
        logged.push(relay);
        logged.push(server);
        logged.push(client);

        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

        network_a.release().await;
        network_b.release().await;

        docker.write_logs_to_file(&log_dir, logged).await.unwrap();

        let report: log::Report = log::Report::from_dir(&log_dir).unwrap();
        assert!(report.is_proof_of_startup());
        assert!(report.is_proof_of_cohesion());
        assert!(report.is_proof_of_connectivity_persistence(2));
        assert!(report.is_proof_of_grpc_interaction());
    }
}


#[tokio::test]
async fn t() {
    let log_dir = std::path::PathBuf::new()
        .join("tests")
        .join("log")
        .join("nat_hard");

    let docker = bollard::Docker::connect_with_local_defaults().unwrap();

    docker.load_built_tar_image_from_ws_target_dir().await.unwrap();
    
    let wan = "wan";
    let lan_a = "lan_b";
    let lan_b = "lan_b";

    net::Ext::create_network(&docker, wan, None).await.unwrap();
    net::Ext::create_network(&docker, lan_a, None).await.unwrap();
    net::Ext::create_network(&docker, lan_b, None).await.unwrap();

    let client_router = testcontainers::GenericImage::new("alpine", "latest")
        .with_cmd(["sh", "-c", "apk add --no-cache iproute2 && sleep infinity"])
        .with_privileged(true)
        .with_container_name("client_router")
        .start()
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(60)).await;

    let client_router = router::Image::new(&docker, client_router);
    let client_router = router::Router::new(
        client_router,
        lan_a.to_owned(),
        wan.to_owned()
    )
    .await
    .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(60)).await;

    let client_router_lan_ip = client_router.ip(lan_a).await.unwrap().unwrap();
    let client_router_lan_ip_str = client_router_lan_ip.to_string();

    let client_router_public_ip = client_router.ip(wan).await.unwrap().unwrap();
    let client_router_public_ip_str = client_router_public_ip.to_string();

    let server_router = testcontainers::GenericImage::new("alpine", "latest")
        .with_cmd(["sh", "-c", "apk add --no-cache iproute2 && sleep infinity"])
        .with_privileged(true)
        .with_container_name("server_router")
        .start()
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(60)).await;

    let server_router = router::Image::new(&docker, server_router);
    let server_router = router::Router::new(
        server_router,
        lan_b.to_owned(),
        wan.to_owned()
    )
    .await
    .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(60)).await;

    let server_router_lan_ip = server_router.ip(lan_b).await.unwrap().unwrap();
    let server_router_lan_ip_str = server_router_lan_ip.to_string();

    let server_router_public_ip = server_router.ip(wan).await.unwrap().unwrap();
    let server_router_public_ip_str = server_router_public_ip.to_string();

    let udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
    let tcp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

    let bootstrap = testcontainers::GenericImage::new("node", "latest")
        .with_exposed_port(udp_port)
        .with_exposed_port(tcp_port)
        .with_privileged(true)
        .with_cmd(["./bootstrap"])
        .with_container_name("bootstrap")
        .start()
        .await
        .unwrap();

    let bootstrap = router::Image::new(&docker, bootstrap);

    client_router.register_public(&bootstrap).await.unwrap();
    server_router.register_public(&bootstrap).await.unwrap();

    let bootstrap_public_ip = bootstrap.ip(wan).await.unwrap().unwrap();
    let bootstrap_public_addr = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_public_ip);

    let bootstrap_grpc_port = bootstrap.get_host_port_ipv4(8080).await.unwrap();
    let bootstrap_grpc_endpoint = format!("http://127.0.0.1:{}", bootstrap_grpc_port);

    let mut bootstrap_grpc = proto::node_client::NodeClient::connect(bootstrap_grpc_endpoint).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let bootstrap_peer_id_request = proto::PeerIdRequest {

    };

    let bootstrap_peer_id_response = bootstrap_grpc.peer_id(bootstrap_peer_id_request).await.unwrap();
    let bootstrap_peer_id_response = bootstrap_peer_id_response.into_inner();
    let bootstrap_peer_id = bootstrap_peer_id_response.peer_id;

    let relay = testcontainers::GenericImage::new("node", "latest")
        .with_exposed_port(udp_port)
        .with_exposed_port(tcp_port)
        .with_privileged(true)
        .with_cmd(["./relay"])
        .with_container_name("relay")
        .start()
        .await
        .unwrap();

    let relay = router::Image::new(&docker, relay);
    
    client_router.register_public(&relay).await.unwrap();
    server_router.register_public(&relay).await.unwrap();

    let relay_public_ip = relay.ip(wan).await.unwrap().unwrap();

    let relay_grpc_port = relay.get_host_port_ipv4(8080).await.unwrap();
    let relay_grpc_endpoint = format!("http://127.0.0.1:{}", relay_grpc_port);

    let mut relay_grpc = proto::node_client::NodeClient::connect(relay_grpc_endpoint).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let relay_peer_id_request = proto::PeerIdRequest {

    };

    let relay_peer_id_response = relay_grpc.peer_id(relay_peer_id_request).await.unwrap();
    let relay_peer_id_response = relay_peer_id_response.into_inner();
    let relay_peer_id = relay_peer_id_response.peer_id;

    let server = testcontainers::GenericImage::new("node", "latest")
        .with_exposed_port(udp_port)
        .with_exposed_port(tcp_port)
        .with_privileged(true)
        .with_cmd(["./server"])
        .with_container_name("server")
        .start()
        .await
        .unwrap();

    let server = router::Image::new(&docker, server);

    server_router.register(&server).await.unwrap();

    let server_grpc_port = server.get_host_port_ipv4(8080).await.unwrap();
    let server_grpc_endpoint = format!("http://127.0.0.1:{}", server_grpc_port);

    let mut server_grpc = proto::node_client::NodeClient::connect(server_grpc_endpoint).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let server_peer_id_request = proto::PeerIdRequest {

    };

    let server_peer_id_response = server_grpc.peer_id(server_peer_id_request).await.unwrap();
    let server_peer_id_response = server_peer_id_response.into_inner();
    let server_peer_id = server_peer_id_response.peer_id;

    let server_addr_via_relay = format!("/ip4/{}/udp/4001/quic-v1/p2p/{}/p2p-circuit/p2p/{}", bootstrap_public_ip, bootstrap_peer_id, server_peer_id);

    let client = testcontainers::GenericImage::new("node", "latest")
        .with_exposed_port(udp_port)
        .with_exposed_port(tcp_port)
        .with_network(lan_a)
        .with_privileged(true)
        .with_cmd([
            &format!("./{} --dial {}", "client", bootstrap_public_addr)
        ])
        .with_container_name("client")
        .start()
        .await
        .unwrap();

    let client = router::Image::new(&docker, client);

    client_router.register(&client).await.unwrap();

    let client_grpc_port = client.get_host_port_ipv4(8080).await.unwrap();
    let client_grpc_endpoint = format!("http://127.0.0.1:{}", client_grpc_port);

    let mut client_grpc = proto::node_client::NodeClient::connect(client_grpc_endpoint).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let client_peer_id_request = proto::PeerIdRequest {

    };

    let client_peer_id_response = client_grpc.peer_id(client_peer_id_request).await.unwrap();
    let client_peer_id_response = client_peer_id_response.into_inner();
    let client_peer_id = client_peer_id_response.peer_id;

    let client_dial_request = proto::DialRequest {
        addr: server_addr_via_relay
    };

    client_grpc.dial(client_dial_request).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    let mut logged: Vec<_> = vec![];
    logged.push(bootstrap);
    logged.push(relay);
    logged.push(server);
    logged.push(server_router.image);
    logged.push(client);
    logged.push(client_router.image);

    tokio::fs::remove_dir_all(&log_dir).await.ok();
    tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

    net::Ext::remove_network(&docker, wan).await.unwrap();
    net::Ext::remove_network(&docker, lan_a).await.unwrap();
    net::Ext::remove_network(&docker, lan_b).await.unwrap();

    docker.write_logs_to_file(&log_dir, logged.into_iter().map(|x| x.x).collect()).await.unwrap();

    let report: log::Report = log::Report::from_dir(&log_dir).unwrap();
    // assert!(report.is_proof_of_startup());
    // assert!(report.is_proof_of_cohesion());
    // assert!(report.is_proof_of_connectivity_persistence(2));
    // assert!(report.is_proof_of_grpc_interaction());
}

struct NatHard;

#[async_trait::async_trait]
impl Test for NatHard {
    async fn run(&self, docker: &bollard::Docker) {

    }
}

struct Discovery;

#[async_trait::async_trait]
impl Test for Discovery {
    async fn run(&self, docker: &bollard::Docker) {
        let log_dir: std::path::PathBuf = std::path::PathBuf::new()
            .join("tests")
            .join("log")
            .join("discovery");

        let network: &str = "an";
        let network_udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let network_tpc_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

        docker.reset().await.ok();
        docker.reset_network(network).await.ok();

        let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(network_udp_port)
            .with_exposed_port(network_tpc_port)
            .with_cmd(["./bootstrap"])
            .with_network(network)
            .start()
            .await
            .unwrap();
        let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.unwrap();
        let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

        let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(network_udp_port)
            .with_exposed_port(network_tpc_port)
            .with_cmd(["./relay", "--dial", &bootstrap_addr])
            .with_network(network)
            .start()
            .await
            .unwrap();

        let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(network_udp_port)
            .with_exposed_port(network_tpc_port)
            .with_cmd(["./server", "--dial", &bootstrap_addr])
            .with_network(network)
            .start()
            .await
            .unwrap();

        let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(network_udp_port)
            .with_exposed_port(network_tpc_port)
            .with_cmd(["./client", "--dial", &bootstrap_addr])
            .with_network(network)
            .start()
            .await
            .unwrap();

        let containers: Vec<_> = vec![
            bootstrap,
            relay,
            server,
            client
        ];

        tokio::time::sleep(std::time::Duration::from_mins(9)).await;
        
        let port = containers
            .get(3)
            .unwrap()
            .get_host_port_ipv4(8080)
            .await
            .unwrap();
        let endpoint = format!("http://127.0.0.1:{}", port);
        let mut client = proto::node_client::NodeClient::connect(endpoint).await.unwrap();
        
        let request = tonic::Request::new(proto::PingRequest{ msg: "Hello".to_owned() });
        let response = client.ping(request).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_mins(1)).await;

        docker.reset().await.ok();
        docker.reset_network(network).await.ok();

        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");

        docker.write_logs_to_file(&log_dir, containers).await.unwrap();
    }
}
 
struct Simulation;

#[async_trait::async_trait]
impl Test for Simulation {
    async fn run(&self, docker: &bollard::Docker) {
        let log_dir: std::path::PathBuf = std::path::PathBuf::new()
            .join("tests")
            .join("log")
            .join("sim");
        
        let mut containers: Vec<_> = vec![];

        let network_a_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::A.to_owned(),
            driver: None,
            scope: None,
            internal: Some(false),
            attachable: Some(false),
            ingress: Some(false),
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: Some(true),
            enable_ipv6: Some(false),
            options: None,
            labels: None
        };

        let network_b_conf: bollard::secret::NetworkCreateRequest = bollard::secret::NetworkCreateRequest {
            name: network::B.to_owned(),
            driver: None,
            scope: None,
            internal: Some(false),
            attachable: Some(false),
            ingress: Some(false),
            config_from: None,
            config_only: None,
            ipam: None,
            enable_ipv4: Some(true),
            enable_ipv6: Some(false),
            options: None,
            labels: None
        };

        let network_a: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_a_conf)
            .name(network::A)
            .reserve()
            .await
            .unwrap();
    
        let network_b: network::Network = network::Network::builder()
            .docker(docker)
            .docker_network_conf(network_b_conf)
            .name(network::B)
            .reserve()
            .await
            .unwrap();
        
        let udp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Udp(4001);
        let tcp_port: testcontainers::core::ContainerPort = testcontainers::core::ContainerPort::Tcp(8080);

        let bootstrap: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./bootstrap"])
            .with_network(network::A)
            .with_network(network::B)
            .start()
            .await
            .expect("successful container launch");

        let bootstrap_ip: std::net::IpAddr = bootstrap.get_bridge_ip_address().await.expect("bridge ip addr");
        let bootstrap_addr: String = format!("/ip4/{}/udp/4001/quic-v1", bootstrap_ip);

        let bootstrap_container_id: &str = bootstrap.id();

        let bootstrap_grpc_port: u16 = bootstrap.get_host_port_ipv4(8080).await.unwrap();
        let bootstrap_grpc_endpoint: String = format!("http://127.0.0.1:{}", bootstrap_grpc_port);
        
        let mut bootstrap_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(bootstrap_grpc_endpoint).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let bootstrap_peer_id_request: proto::PeerIdRequest = proto::PeerIdRequest { };
        let bootstrap_peer_id: String = bootstrap_grpc
            .peer_id(bootstrap_peer_id_request)
            .await
            .unwrap()
            .into_inner()
            .peer_id;

        let bootstrap_addr_a: String = format!("/dns4/bootstrap/ip4/0.0.0.0/udp/4001/quic-v1/p2p/{}", bootstrap_peer_id);
        let bootstrap_addr_b: String = format!("/dns4/bootstrap/ip4/0.0.0.0/udp/4001/quic-v1/p2p/{}", bootstrap_peer_id);

        containers.push(bootstrap);

        for _ in 0..=16 {
            let cmd: &str = if rand::random::<f32>() < 0.25 {
                "./malicious_relay"
            } else {
                "./relay"
            };

            let relay: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
                .with_exposed_port(udp_port)
                .with_exposed_port(tcp_port)
                .with_cmd([cmd, "--dial", &bootstrap_addr])
                .with_network(network::A)
                .with_network(network::B)
                .start()
                .await
                .expect("successful container launch");

            containers.push(relay);
        }

        let server: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./server", "--dial", &bootstrap_addr])
            .with_network(network::B)
            .start()
            .await
            .expect("successful container launch");

        let server_ip: std::net::IpAddr = server.get_bridge_ip_address().await.expect("bridge ip addr");
        let server_addr: String = format!("/ip4/{}/udp/4001/quic-v1", server_ip);

        containers.push(server);

        let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("node", "latest")
            .with_exposed_port(udp_port)
            .with_exposed_port(tcp_port)
            .with_cmd(["./client", "--dial", &bootstrap_addr])
            .with_network(network::A)
            .start()
            .await
            .expect("successful container launch");

        let client_grpc_port: u16 = client.get_host_port_ipv4(8080).await.expect("host port ipv4");
        let client_gprc_endpoint: String = format!("http://127.0.0.1:{}", client_grpc_port);
        let mut client_grpc: proto::node_client::NodeClient<_> = proto::node_client::NodeClient::connect(client_gprc_endpoint).await.expect("successful grpc client");

        containers.push(client);

        for _ in 0..=8 {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;

            let client_request: proto::DialRequest = proto::DialRequest {
                addr: server_addr.to_owned()    
            };

            client_grpc.dial(client_request).await.expect("successful dial");
        }

        tokio::time::sleep(std::time::Duration::from_mins(10)).await;

        // ... clean up ...
        network_a.release().await;
        network_b.release().await;

        // ... proof ...
        tokio::fs::remove_dir_all(&log_dir).await.ok();
        tokio::fs::create_dir_all(&log_dir).await.expect("unable to create logs directory");
        
        docker.write_logs_to_file(&log_dir, containers).await.unwrap();
        
        let report: log::Report = log::Report::from_dir(&log_dir).unwrap();
        assert!(report.is_proof_of_startup());
        assert!(report.is_proof_of_cohesion());
        assert!(report.is_proof_of_stability(85));
        assert!(report.is_proof_of_connectivity_persistence(6));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn end_to_end() {
    let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults().unwrap();
    let mut harness: Harness = Harness::new(docker);
    //harness.add_test(NatEasy);
    harness.add_test(NatHard);
    //harness.add_test(Discovery);
    //harness.add_test(Simulation);
    harness.launch().await;
}

async fn wait_for_grpc(endpoint: String) -> proto::node_client::NodeClient<tonic::transport::Channel> {
    loop {
        match proto::node_client::NodeClient::connect(endpoint.clone()).await {
            Ok(client) => return client,
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(500)).await,
        }
    }
}