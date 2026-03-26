use super::*;

#[derive(Debug)]
#[derive(getset::Getters)]
pub struct Network<'a> {
    interface: &'a bollard::Docker,
    #[getset(get = "pub")]
    name: String
}

impl<'a> Network<'a> {
    pub async fn new(interface: &'a bollard::Docker, name: String) -> Result<Self> {
        interface.create_network(bollard::secret::NetworkCreateRequest {
            name: name.to_owned(),
            driver: Some(String::from("bridge")),
            internal: Some(false),
            ipam: None,
            enable_ipv4: Some(true),
            enable_ipv6: Some(false),
            ..Default::default()
        })
        .await?;
        Ok(Self { interface, name })
    }

    pub async fn from_network_create_request(interface: &'a bollard::Docker, configuration: bollard::secret::NetworkCreateRequest) -> Result<Self> {
        let name: String = configuration.name.to_owned();
        interface.create_network(configuration).await?;
        Ok(Self { interface, name })
    }

    pub async fn has<'b, T>(&self, container: &container::Container<'b, T>) -> Result<bool>
    where
        T: testcontainers::Image {
        let container_id: &str = container.id();
        let inspect: bollard::secret::ContainerInspectResponse = self.interface.inspect_container(container_id, None).await?;
        let Some(configuration) = inspect.network_settings else {
            return Ok(false)
        };
        let Some(network_to_endpoint_configuration) = configuration.networks else {
            return Ok(false)
        };
        let found: bool = network_to_endpoint_configuration.contains_key(&self.name);
        Ok(found)
    }

    pub async fn cidr(&self) -> Result<Option<cidr::Cidr>> {
        let inspect: bollard::secret::NetworkInspect = self.interface.inspect_network(&self.name, None).await?;
        let Some(ipam) = inspect.ipam else {
            return Ok(None)
        };
        let Some(ipam_configs) = ipam.config else {
            return Ok(None)
        };
        let Some(ipam_config) = ipam_configs.first() else {
            return Ok(None)
        };
        let Some(subnet) = &ipam_config.subnet else {
            return Ok(None)
        };
        let parts: Vec<&str> = subnet.split('/').collect();
        let addr: std::net::Ipv4Addr = parts[0].parse()?;
        let mask: u8 = parts[1].parse()?;
        Ok(Some(cidr::Cidr::new(addr, mask)?))
    }

    pub async fn release(self) -> Result<()> {
        let containers: Vec<_> = self.interface.list_containers(None).await.unwrap_or_default();
        for container in containers {
            let Some(id) = container.id else {
                continue
            };
            let request: bollard::secret::NetworkDisconnectRequest = bollard::secret::NetworkDisconnectRequest {
                container: id,
                force: Some(true)
            };
            self.interface.disconnect_network(&self.name, request).await.ok();
        }
        self.interface.remove_network(&self.name).await.ok();
        Ok(())
    }
}