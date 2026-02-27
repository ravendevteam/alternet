#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Server {
    pub domain: String,
    pub bid: u32,
    pub identity_cache_size: Option<usize>
}

#[bon::bon]
impl Server {
    #[builder]
    pub fn new(
        domain: String,
        bid: u32,
        identity_cache_size: Option<usize>
    ) -> Self {
        Self {
            domain,
            bid,
            identity_cache_size
        }
    }
}