use super::*;

#[derive(Debug)]
pub struct Container<'a, T> 
where
    T: testcontainers::Image {
    interface: &'a bollard::Docker,
    container: testcontainers::ContainerAsync<T>
}

impl<'a, A> Container<'a, A>
where
    A: testcontainers::Image {
    pub fn new(interface: &'a bollard::Docker, container: testcontainers::ContainerAsync<A>) -> Self {
        Self {
            interface,
            container
        }
    }

    pub fn id(&self) -> &str {
        self.container.id()
    }

    pub async fn start(&self) -> Result<()> {
        self.container.start().await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.container.stop().await?;
        Ok(())
    }

    pub async fn host(&self) -> Result<url::Host> {
        let host: url::Host = self.container.get_host().await?;
        Ok(host)
    }

    pub async fn host_ipv4_port(&self, port: u16) -> Result<u16> {
        let port: u16 = self.container.get_host_port_ipv4(port).await?;
        Ok(port)
    }

    pub async fn host_ipv6_port(&self, port: u16) -> Result<u16> {
        let port: u16 = self.container.get_host_port_ipv6(port).await?;
        Ok(port)
    }

    pub async fn ip<'b>(&self, network: &network::Network<'b>) -> Result<Option<std::net::Ipv4Addr>> {
        let name: &str = self.container.id();
        let network_name: &str = network.name();
        let response: bollard::secret::ContainerInspectResponse = self.interface.inspect_container(name, None).await?;        
        if let Some(response) = response.network_settings
        && let Some(response) = response.networks
        && let Some(response) = response.get(network_name)
        && let Some(response) = response.ip_address.as_ref() {
            let ret: std::net::Ipv4Addr = response.parse()?;
            return Ok(Some(ret))
        } 
        Ok(None)
    }

    pub async fn eth<'b>(&self, network: &network::Network<'b>) -> Result<usize> {
        let ip: std::net::Ipv4Addr = self.ip(network).await?.ok_or(anyhow!("no connection"))?;
        let (stdout, _) = self.exec()
            .arg("sh")
            .arg("-c")
            .arg(format!("ip -o addr show | awk '{{print $2, $4}}' | grep '^[^:]* *{}'", ip))
            .send()
            .await?;
        for line in stdout.lines() {
            let Some(start_key) = line.find("eth") else {
                continue
            };            
            let Some(final_key) = line.find(char::is_whitespace) else {
                continue
            };
            let s: String = line[start_key..final_key].to_owned();
            let s_len: usize = s.len();
            let start_key: usize = s.find(char::is_numeric).ok_or(anyhow!(""))?;
            let ret: &str = &s[start_key..s_len];
            let ret: usize = ret.parse()?;
            return Ok(ret);
        }
        Err(anyhow::anyhow!("no interface found for network `{:?}` (ip={})", network, ip))
    }

    pub async fn connect_to<'b>(&self, network: &network::Network<'b>) -> Result<()> {
        let network_name: &str = network.name();
        self.interface.connect_network(network_name, bollard::secret::NetworkConnectRequest {
            container: self.container.id().to_owned(),
            ..Default::default()
        })
        .await?;
        Ok(())
    }

    pub fn exec(&self) -> CommandBuilder<'_, A> {
        CommandBuilder::new(&self.container, Vec::new())
    }

    pub async fn logs(&self) -> Result<(String, String)> {
        let mut stdout_buf: Vec<_> = Vec::new();
        let mut stderr_buf: Vec<_> = Vec::new();
        let mut stdout: std::pin::Pin<_> = self.container.stdout(false);
        let mut stderr: std::pin::Pin<_> = self.container.stderr(false);
        let (read_to_end_out, read_to_end_err) = tokio::join!(
            stdout.read_to_end(&mut stdout_buf),
            stderr.read_to_end(&mut stderr_buf)
        );
        read_to_end_out?;
        read_to_end_err?;
        let stdout: String = String::from_utf8_lossy(&stdout_buf).into_owned();
        let stderr: String = String::from_utf8_lossy(&stderr_buf).into_owned();
        Ok((stdout, stderr))
    }

    pub async fn write_logs_to_file(&self, path: &std::path::Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }    
        let configuration: bollard::query_parameters::LogsOptions = bollard::query_parameters::LogsOptionsBuilder::new()
            .stdout(true)
            .stderr(true)
            .timestamps(true)
            .tail("all")
            .build();
        let id: &str = self.container.id();
        let mut file: tokio::fs::File = tokio::fs::File::create(path).await?;
        let mut stream = self.interface.logs(id, Some(configuration));
        while let Some(Ok(log)) = stream.next().await {
            let bytes = match log {
                bollard::container::LogOutput::StdOut { message } => message,
                bollard::container::LogOutput::StdErr { message } => message,
                bollard::container::LogOutput::Console {
                    message
                } => {
                    message
                },
                _ => continue
            };
            file.write_all(&bytes).await?;
        }
        Ok(())
    }

    pub async fn can_reach(&self, addr: &std::net::Ipv4Addr) -> bool {
        self.exec()
            .arg("ping")
            .arg("-c")
            .arg("1")
            .arg("-W")
            .arg("1")
            .arg(format!("{}", addr))
            .send()
            .await
            .is_ok()
    }

    pub async fn can_reach_tcp_endpoint_on_network<'b, 'c, B>(&self, network: &network::Network<'b>, container: &container::Container<'c, A>, port: u16) -> Result<()>
    where
        B: testcontainers::Image {
        let addr: std::net::Ipv4Addr = container.ip(network).await?.ok_or(anyhow!("no connection"))?;
        self.can_reach_tcp_endpoint(&addr, port).await?;
        Ok(())
    }

    pub async fn can_reach_tcp_endpoint(&self, addr: &std::net::Ipv4Addr, port: u16) -> Result<()> {
        self.exec()
            .arg("nc")
            .arg("-z")
            .arg("-w")
            .arg("2")
            .arg(format!("{}", addr))
            .arg(format!("{}", port))
            .send()
            .await?;
        Ok(())
    }

    pub async fn can_reach_udp_endpoint_on_network<'b, 'c, B>(&self, network: &network::Network<'b>, container: &container::Container<'c, A>, port: u16) -> Result<()> 
    where
        B: testcontainers::Image {
        let addr: std::net::Ipv4Addr = container.ip(network).await?.ok_or(anyhow!("no connection"))?;
        self.can_reach_udp_endpoint(&addr, port).await?;
        Ok(())
    }

    pub async fn can_reach_udp_endpoint(&self, addr: &std::net::Ipv4Addr, port: u16) -> Result<()> {
        self.exec()
            .arg("nc")
            .arg("-z")
            .arg("-u")
            .arg("-w")
            .arg("2")
            .arg(format!("{}", addr))
            .arg(format!("{}", port))
            .send()
            .await?;
        Ok(())
    }

    pub async fn open_tcp_port(&self, port: u16) -> Result<()> {
        self.exec()
            .arg("nc")
            .arg("-l")
            .arg("-p")
            .arg(format!("{}", port))
            .send()
            .await?;
        Ok(())
    }

    pub async fn open_udp_port(&self, port: u16) -> Result<()> {
        self.exec()
            .arg("nc")
            .arg("-u")
            .arg("-l")
            .arg("-p")
            .arg(format!("{}", port))
            .send()
            .await?;
        Ok(())
    }

    pub async fn close_tcp_port(&self, port: u16) -> Result<()> {
        self.exec()
            .arg("fuser")
            .arg("-k")
            .arg(format!("{}/tcp", port))
            .send()
            .await?;
        Ok(())
    }

    pub async fn close_udp_port(&self, port: u16) -> Result<()> {
        self.exec()
            .arg("fuser")
            .arg("-k")
            .arg(format!("{}/udp", port))
            .send()
            .await?;
        Ok(())
    }

    pub async fn close_all_ports(&self) -> Result<()> {
        self.exec().args(vec!["pkill", "nc"]).send().await?;
        Ok(())
    }

    pub async fn set_reachable_route_to_network<'b>(&self, network: &network::Network<'b>, gateway: &std::net::Ipv4Addr) -> Result<()> {
        if !self.can_reach(gateway).await {
            return Err(anyhow::anyhow!("unreachable"))
        }
        self.set_route_to_network(network, gateway).await
    }

    pub async fn set_reachable_default_gateway(&self, gateway: &std::net::Ipv4Addr) -> Result<()> {
        if !self.can_reach(gateway).await {
            return Err(anyhow::anyhow!("unreachable"))
        }
        self.set_default_gateway(gateway).await
    }

    pub async fn set_route_to_network<'b>(&self, network: &network::Network<'b>, gateway: &std::net::Ipv4Addr) -> Result<()> {
        let network_cidr: cidr::Cidr = network.cidr().await?.ok_or(anyhow::anyhow!(""))?;
        self.exec().args(vec!["ip", "route", "add", &format!("{}", network_cidr), "via", &format!("{}", gateway)]).send().await?;
        Ok(())
    }

    pub async fn set_default_gateway(&self, gateway: &std::net::Ipv4Addr) -> Result<()> {
        self.exec().args(vec!["ip", "route", "del", "default"]).send().await?;
        self.exec().args(vec!["ip", "route", "add", "default", "via", &format!("{}", gateway)]).send().await?;
        Ok(())
    }

    pub async fn release(self) -> Result<()> {
        self.stop().await?;
        self.container.rm().await?;
        Ok(())
    }
}

pub struct CommandBuilder<'a, T> 
where
    T: testcontainers::Image {
    container: &'a testcontainers::ContainerAsync<T>,
    tokens: Vec<String>
}

impl<'a, T> CommandBuilder<'a, T> 
where
    T: testcontainers::Image {
    pub fn new(container: &'a testcontainers::ContainerAsync<T>, tokens: Vec<String>) -> Self {
        Self { container, tokens }
    }

    pub fn arg(mut self, token: impl Into<String>) -> Self {
        let token: String = token.into();
        self.tokens.push(token);
        self
    }

    pub fn args(mut self, tokens: Vec<impl Into<String>>) -> Self {
        let tokens: Vec<_> = tokens.into_iter().map(|token| token.into()).collect();
        self.tokens.extend(tokens);
        self
    }

    pub async fn send(self) -> Result<(String, String)> {
        let mut outcome: testcontainers::core::ExecResult = self.container.exec(testcontainers::core::ExecCommand::new(self.tokens)).await?;
        let code: i64;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let Some(code_out) = outcome.exit_code().await? else {
                continue
            };
            code = code_out;
            break
        }
        let stdout: Vec<_> = outcome.stdout_to_vec().await?;
        let stderr: Vec<_> = outcome.stderr_to_vec().await?;
        let stdout: String = String::from_utf8_lossy(&stdout).into_owned();
        let stderr: String = String::from_utf8_lossy(&stderr).into_owned();
        if code == 0 {
            Ok((stdout, stderr))
        } else {
            Err(anyhow::anyhow!("(code={})\n{}\n{}", code, stdout, stderr))
        }
    }
}