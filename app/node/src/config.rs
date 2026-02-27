use super::*;

pub mod bootstrap;
pub mod client;
pub mod mode;
pub mod relay;
pub mod server;

#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Config {
    pub mode: mode::Mode,
    pub bootstrap: Option<bootstrap::Bootstrap>,
    pub client: Option<client::Client>,
    pub server: Option<server::Server>,
    pub relay: Option<relay::Relay>
}

#[bon::bon]
impl Config {
    #[builder]
    pub fn new(
        mode: mode::Mode,
        bootstrap: Option<bootstrap::Bootstrap>,
        client: Option<client::Client>,
        server: Option<server::Server>,
        relay: Option<relay::Relay>
    ) -> Self {
        Self {
            mode,
            bootstrap,
            client,
            server,
            relay
        }
    }

    pub fn from_toml_at(path: &std::path::Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None)
        }
        let content: String = std::fs::read_to_string(path)?;
        let new: Self = toml::from_str(&content)?;
        Ok(Some(new))
    }

    pub fn from_toml() -> Result<Option<Self>> {
        let path: std::path::PathBuf = std::env::current_dir()?;
        let path: std::path::PathBuf = path.join("an.toml");
        Self::from_toml_at(&path)
    }
}