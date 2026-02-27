#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Bootstrap {
    pub dial: Option<Vec<libp2p::Multiaddr>>
}

#[bon::bon]
impl Bootstrap {
    #[builder]
    pub fn new(dial: Option<Vec<libp2p::Multiaddr>>) -> Self {
        Self {
            dial
        }
    }
}