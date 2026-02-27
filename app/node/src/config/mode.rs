#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Server,
    Client,
    Relay
}