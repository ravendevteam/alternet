#[derive(Debug)]
#[derive(Clone)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Client {
    #[serde(rename = "identity-cache-size")]
    pub identity_cache_size: Option<usize>
}

#[bon::bon]
impl Client {
    #[builder]
    pub fn new(identity_cache_size: Option<usize>) -> Self {
        Self {
            identity_cache_size
        }
    }
}