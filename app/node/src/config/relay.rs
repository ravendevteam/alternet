#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Relay {
    pub identity_cache_size: Option<usize>
}

#[bon::bon]
impl Relay {
    #[builder]
    pub fn new(
        identity_cache_size: Option<usize>
    ) -> Self {
        Self {
            identity_cache_size
        }
    }
}