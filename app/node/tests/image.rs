#![allow(async_fn_in_trait)]

mod exec_result_ext {
    use super::*;

    pub trait ExecResultExt {
        async fn success(&mut self) -> Result<bool>;
        async fn lines(&mut self) -> Vec<String>;
        async fn read(&mut self) -> String;
    }

    impl ExecResultExt for testcontainers::core::ExecResult {
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

mod network_ext {
    use super::*;

    pub type Docker = bollard::Docker;

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

    pub trait NetworkExt {
        async fn create_network(&self, name: &str, configuration: Option<Configuration>) -> Result<()>;
        async fn remove_network(&self, name: &str) -> Result<()>;
    }

    impl NetworkExt for Docker {
        async fn create_network(&self, name: &str, configuration: Option<Configuration>) -> Result<()> {
            <Self as NetworkExt>::remove_network(self, name).await.ok();
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

mod image {
    use super::*;
    use exec_result_ext::ExecResultExt as _;

    #[derive(Debug)]
    #[derive(thiserror::Error)]
    pub enum Error {
        #[error("{}", 0)]
        NotConnection,
        #[error("{}", 0)]
        NoInterfaceOnNetwork(String)
    }

    #[derive(Debug)]
    #[derive(derive_more::Deref)]
    #[derive(derive_more::DerefMut)]
    #[derive(getset::Getters)]
    pub struct Image<'a> {
        #[getset(get = "pub")]
        docker: &'a bollard::Docker,
        #[deref]
        #[deref_mut]
        #[getset(get = "pub")]
        component: testcontainers::ContainerAsync<testcontainers::GenericImage>
    }

    // model
    // ... > router > internet < router < ...

    impl<'a> Image<'a> {
        pub fn new(docker: &'a bollard::Docker, component: testcontainers::ContainerAsync<testcontainers::GenericImage>) -> Image<'a> {
            Self {
                docker,
                component
            }
        }

        pub async fn ip(&self, network: &str) -> Result<Option<String>> {
            let name: &str = self.id();
            let response: bollard::secret::ContainerInspectResponse = self.docker.inspect_container(name, None).await?;        
            if let Some(response) = response.network_settings
            && let Some(response) = response.networks
            && let Some(response) = response.get(network)
            && let Some(response) = response.ip_address.as_ref() {
                let ret: String = response.to_owned();
                return Ok(Some(ret))
            } 
            Ok(None)
        }

        pub async fn eth(&self, network: &str) -> Result<String> {
            let ip: String = self.ip(network).await?.ok_or(Error::NotConnection)?;
            // Error: failed to initialize exec command: Docker responded with status code 409: container 7390b2674e7ba8f8974b261c224a445ea2837efd38cdfd77eb9c6ff1d64c69ee is not running
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
            Err(anyhow::anyhow!("no interface found for network `{}` (ip={})", network, ip))
        }

        pub async fn exec_wait(&self, cmd: Vec<&str>) -> Result<String> {
            // ffs testcontainers why make me do this : ( - not cool
            let cmd_copy_a: Vec<_> = cmd.to_owned();
            let cmd_copy_b: Vec<_> = cmd.to_owned();
            let mut outcome: testcontainers::core::ExecResult = self.exec(testcontainers::core::ExecCommand::new(cmd_copy_a)).await?;
            if !outcome.success().await? {
                let output: String = outcome.read().await;
                return Err(anyhow::anyhow!("command failure: `{:?}`: {}", cmd_copy_b, output))
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
}




mod router {
    use super::*;
    use anyhow::Ok;
    use testcontainers::ImageExt as _;
    use testcontainers::runners::AsyncRunner as _;

    #[derive(Debug)]
    #[derive(derive_more::Deref)]
    #[derive(derive_more::DerefMut)]
    #[derive(getset::Getters)]
    pub struct Isp<'a> {
        #[deref]
        #[deref_mut]
        image: image::Image<'a>,
        wan: String
    }

    impl<'a> Isp<'a> {
        pub async fn new(docker: &'a bollard::Docker, wan: String) -> Result<Self> {
            let image: testcontainers::ContainerAsync<testcontainers::GenericImage> = testcontainers::GenericImage::new("alpine", "latest")
                .with_privileged(true)
                .with_cmd(vec!["tail", "-f", "/dev/null"])
                .with_startup_timeout(std::time::Duration::from_mins(1))
                .start()
                .await?;
            let image: image::Image = image::Image::new(&docker, image);
            let cmds: [_; _] = [
                vec!["apk", "add", "--no-cache", "iproute2"],
                vec!["sysctl", "-w", "net.ipv4.ip_forward=1"]
            ];
            for cmd in cmds {
                image.exec_wait(cmd).await?;
            }
            image.connect_to_network(&wan, None).await?;
            Ok(Self { image, wan })
        }
        
        pub async fn wan_ip(&self) -> Result<String> {
            let ret: String = self.ip(&self.wan).await?.ok_or(anyhow::anyhow!("no connection"))?;
            Ok(ret)
        }

        pub async fn connect_router(&self, router: &image::Image<'a>) -> Result<()> {
            let wan_ip: String = self.wan_ip().await?;
            let router_wan_eth: String = router.eth(&self.wan).await?;
            router.exec_wait(vec!["ip", "route", "del", "default"]).await?;
            router.exec_wait(vec!["ip", "route", "add", "default", "via", &wan_ip, "dev", &router_wan_eth]).await?;
            Ok(())
        }

        pub async fn connect_route_to_router(&self, router_wan_ip: &str, lan_subnet: &str) -> Result<()> {
            self.image.exec_wait(vec!["ip", "route", "add", lan_subnet, "via", router_wan_ip]).await?;
            Ok(())
        }

        pub async fn connect_public_service(&self, service: &image::Image<'a>) -> Result<()> {
            let wan: String = self.wan.to_owned();
            let wan_ip: String = self.wan_ip().await?;
            
            service.connect_to_network(&self.wan, None).await?;
            
            let service_wan_eth: String = service.eth(&wan).await?;

            service.exec_wait(vec!["ip", "route", "del", "default"]).await?;
            service.exec_wait(vec!["ip", "route", "add", "default", "via", &wan_ip, "dev", &service_wan_eth]).await?;
            
            Ok(())
        }
    }

    #[derive(Debug)]
    #[derive(derive_more::Deref)]
    #[derive(derive_more::DerefMut)]
    #[derive(getset::Getters)]
    pub struct SnatMasquerade<'a> {
        #[deref]
        #[deref_mut]
        #[getset(get = "pub")]
        image: image::Image<'a>,
        #[getset(get = "pub")]
        lan: String,
        #[getset(get = "pub")]
        wan: String
    }

    impl<'a> SnatMasquerade<'a> {
        pub async fn new(
            docker: &'a bollard::Docker,
            lan: String,
            wan: String
        ) -> Result<Self> {
            let image: testcontainers::ContainerAsync<testcontainers::GenericImage> = testcontainers::GenericImage::new("alpine", "latest")
                .with_privileged(true)
                .with_cmd(vec!["tail", "-f", "/dev/null"])
                .with_startup_timeout(std::time::Duration::from_mins(1))
                .start()
                .await?;
            let image: image::Image = image::Image::new(docker, image);
            image.exec_wait(vec!["apk", "add", "--no-cache", "iptables"]).await?;
            image.exec_wait(vec!["apk", "add", "--no-cache", "iproute2"]).await?;
            image.connect_to_network(&wan, None).await?;
            image.connect_to_network(&lan, None).await?;
            let image_lan_ip: String = image.ip(&lan).await?.ok_or(anyhow::anyhow!("no connection to lan"))?;
            let image_wan_ip: String = image.ip(&wan).await?.ok_or(anyhow::anyhow!("no connection to wan"))?;
            let image_lan_eth: String = image.eth(&lan).await?;
            let image_wan_eth: String = image.eth(&wan).await?;
            let cmds: [_; _] = [
                vec!["sysctl", "-w", "net.ipv4.ip_forward=1"],
                vec!["sysctl", "-w", "net.ipv4.conf.all.rp_filter=0"],
                vec!["sysctl", "-w", "net.ipv4.conf.default.rp_filter=0"],
                vec!["iptables", "-F"],
                vec!["iptables", "-t", "nat", "-F"],
                vec!["iptables", "-t", "nat", "-A", "POSTROUTING", "-o", &image_wan_eth, "-j", "MASQUERADE"],
                vec!["iptables", "-A", "FORWARD", "-p", "icmp", "-j", "ACCEPT"],
                vec!["iptables", "-A", "FORWARD", "-m", "conntrack", "--ctstate", "RELATED,ESTABLISHED", "-j", "ACCEPT"],
                vec!["iptables", "-A", "FORWARD", "-i", &image_lan_eth, "-o", &image_wan_eth, "-j", "ACCEPT"],
                vec!["iptables", "-A", "INPUT", "-i", "lo", "-j", "ACCEPT"],
                vec!["iptables", "-A", "INPUT", "-m", "conntrack", "--ctstate", "RELATED,ESTABLISHED", "-j", "ACCEPT"],
                vec!["iptables", "-A", "INPUT", "-i", &image_lan_eth, "-p", "tcp", "--dport", "22", "-j", "ACCEPT"]
            ];
            for cmd in cmds {
                image.exec_wait(cmd).await?;
            }
            let new: Self = Self {
                image,
                lan,
                wan
            };
            Ok(new)
        }

        pub async fn lan_ip(&self) -> Result<String> {
            let ret: String = self.ip(&self.lan).await?.ok_or(anyhow::anyhow!("no connection to lan"))?;
            Ok(ret)
        }

        pub async fn wan_ip(&self) -> Result<String> {
            let ret: String = self.ip(&self.wan).await?.ok_or(anyhow::anyhow!("no connection to wan"))?;
            Ok(ret)
        }

        pub async fn lan_eth(&self) -> Result<String> {
            self.eth(&self.lan).await
        }

        pub async fn wan_eth(&self) -> Result<String> {
            self.eth(&self.wan).await
        }

        pub async fn forward_port(
            &self,
            external_port: u16,
            internal_ip: std::net::Ipv4Addr,
            internal_port: u16,
            protocol: &str
        ) -> Result<()> {
            self.image.exec_wait(vec![
                "iptables", 
                "--table", "nat", 
                "--append", "PREROUTING",
                "--protocol", protocol,
                "--dport", &external_port.to_string(),
                "--jump", "DNAT",
                "--to-destination", &format!("{}:{}", internal_ip, internal_port)
            ]).await?;
            self.image.exec_wait(vec![
                "iptables", 
                "--append", "FORWARD",
                "--protocol", protocol,
                "--destination", &internal_ip.to_string(),
                "--dport", &internal_port.to_string(),
                "--match", "state", 
                "--state", "NEW,ESTABLISHED,RELATED",
                "--jump", "ACCEPT"
            ]).await?;
            Ok(())
        }

        pub async fn add_local(&self, client: &image::Image<'a>) -> Result<()> {
            let lan_ip: String = self.lan_ip().await?.to_string();
            client.connect_to_network(&self.lan, None).await?;
            let client_lan_eth: String = client.eth(&self.lan).await?;
            let cmds: [_; _] = [
                vec!["apk", "add", "--no-cache", "ethtool"],
                vec!["ethtool", "-K", &client_lan_eth, "tx", "off"],
                vec!["sh", "-c", "ip route show | grep default | x86_64-linux-gnu-awk '{print $3}' | xargs -I {} ip route del default via {}"],
                vec!["ip", "link", "set", &client_lan_eth, "up"],
                vec!["ip", "route", "del", "default"],
                vec!["ip", "route", "add", "default", "via", &lan_ip, "dev", &client_lan_eth]
            ];
            for cmd in cmds {
                client.exec_wait(cmd).await?;
            }
            Ok(())
        }

        pub async fn add_public(&self, public: &image::Image<'a>) -> Result<()> {
            public.connect_to_network(&self.wan, None).await?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[tokio::test]
        async fn outbound_traffic() -> Result<()> {
            // ------------ ------------------------
            // Client > Router ... Target

            let docker: bollard::Docker = bollard::Docker::connect_with_local_defaults()?;
            
            let lan: &str = "lan";
            let lan_conf: network_ext::Configuration = network_ext::Configuration::builder()
                .name(lan)
                .driver("bridge")
                .enable_ipv4(true)
                .build();

            let wan: &str = "wan";
            let wan_conf: network_ext::Configuration = network_ext::Configuration::builder()
                .name(wan)
                .driver("bridge")
                .enable_ipv4(true)
                .build();

            network_ext::NetworkExt::create_network(&docker, lan, Some(lan_conf)).await?;
            network_ext::NetworkExt::create_network(&docker, wan, Some(wan_conf)).await?;
            
            let router: testcontainers::ContainerAsync<testcontainers::GenericImage> = testcontainers::GenericImage::new("alpine", "latest")
                .with_privileged(true)
                .with_cmd(vec!["tail", "-f", "/dev/null"])
                .with_startup_timeout(std::time::Duration::from_mins(1))
                .start()
                .await?;
            let router: image::Image = image::Image::new(&docker, router);
            
            router.exec_wait(vec!["apk", "add", "--no-cache", "iptables"]).await?;
            router.exec_wait(vec!["apk", "add", "--no-cache", "iproute2"]).await?;

            router.connect_to_network(wan, None).await?;
            router.connect_to_network(lan, None).await?;
            
            let router_lan_ip: String = router.ip(lan).await?.ok_or(anyhow::anyhow!("no connection to lan"))?;
            let router_wan_ip: String = router.ip(wan).await?.ok_or(anyhow::anyhow!("no connection to wan"))?;
            let router_lan_eth: String = router.eth(lan).await?;
            let router_wan_eth: String = router.eth(wan).await?;
            let eth0 = router_lan_eth;
            let eth1 = router_wan_eth;
            let router_cmds: [_; _] = [
                vec!["sysctl", "-w", "net.ipv4.ip_forward=1"],
                //vec!["sysctl", "-w", "net.ipv4.conf.all.rp_filter=0"],
                //vec!["sysctl", "-w", "net.ipv4.conf.default.rp_filter=0"],
                
                vec!["iptables", "-t", "nat", "-A", "POSTROUTING", "-o", &eth1, "-j", "MASQUERADE"],
                vec!["iptables", "-A", "FORWARD", "-i", &eth0, "-o", &eth1, "-m", "state", "--state", "RELATED,ESTABLISHED", "-j", "ACCEPT"],
                vec!["iptables", "-A", "FORWARD", "-i", &eth1, "-o", &eth0, "-j", "ACCEPT"]
                
                //vec!["iptables", "-F"],
                //vec!["iptables", "-t", "nat", "-F"],
                //vec!["iptables", "-t", "nat", "-A", "POSTROUTING", "-o", &router_wan_eth, "-j", "MASQUERADE"],
                //vec!["iptables", "-A", "FORWARD", "-p", "icmp", "-j", "ACCEPT"],
                //vec!["iptables", "-A", "FORWARD", "-m", "conntrack", "--ctstate", "RELATED,ESTABLISHED", "-j", "ACCEPT"],
                //vec!["iptables", "-A", "FORWARD", "-i", &router_lan_eth, "-o", &router_wan_eth, "-j", "ACCEPT"],
                //vec!["iptables", "-A", "FORWARD", "-i", &router_wan_eth, "-o", &router_lan_eth, "-j", "ACCEPT"],
                //vec!["iptables", "-A", "INPUT", "-i", "lo", "-j", "ACCEPT"],
                //vec!["iptables", "-A", "INPUT", "-m", "conntrack", "--ctstate", "RELATED,ESTABLISHED", "-j", "ACCEPT"],
                //vec!["iptables", "-A", "INPUT", "-i", &router_lan_eth, "-p", "tcp", "--dport", "22", "-j", "ACCEPT"]
            ];
            for cmd in router_cmds {
                router.exec_wait(cmd).await?;
            }

            let client: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("alpine", "latest")
                .with_cmd(vec!["tail", "-f", "/dev/null"])
                .with_privileged(true)
                .start()
                .await?;
            let client: image::Image = image::Image::new(&docker, client);
            
            client.connect_to_network(lan, None).await?;

            let client_cmds: [_; _] = [
                vec!["apk", "add", "mtr"],
                vec!["ip", "route", "del", "default"],
                vec!["ip", "route", "add", "default", "via", &router_lan_ip]
            ];
            for cmd in client_cmds {
                client.exec_wait(cmd).await?;
            }

            let target: testcontainers::ContainerAsync<_> = testcontainers::GenericImage::new("alpine", "latest")
                .with_cmd(vec!["tail", "-f", "/dev/null"])
                .with_privileged(true)
                .start()
                .await?;            
            let target: image::Image = image::Image::new(&docker, target);
            
            target.connect_to_network(wan, None).await?;

            let target_cmds: [_; _] = [
                vec!["ip", "route", "del", "default"],
                vec!["ip", "route", "add", "default", "via", &router_wan_ip]
            ];
            for cmd in target_cmds {
                target.exec_wait(cmd).await?;
            }

            let target_ip: String = target.ip(wan).await?.ok_or(anyhow::anyhow!("no connection to wan"))?;            

            let ping_output: String = client.exec_wait(vec!["mtr", "--report", "--report-cycles", "10", &format!("{}", target_ip)]).await?;

            // Start: 2026-03-23T18:32:08+0000
            // HOST: ec99f77d9baa                Loss%   Snt   Last   Avg  Best  Wrst StDev
            //  1.|-- stoic_murdock.lan          0.0%    10    0.4   0.4   0.2   1.1   0.3
            //  2.|-- ???                       100.0    10    0.0   0.0   0.0   0.0   0.0
            panic!("{}", ping_output);
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn main() -> Result<()> {

        Ok(())
    }
}

type Result<T> = anyhow::Result<T>;