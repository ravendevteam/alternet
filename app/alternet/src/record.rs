#[derive(Debug)]
#[derive(Clone)]
#[derive(derive_new::new)]
#[derive(bincode::Encode)]
#[derive(bincode::Decode)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct Domain {
    name: heapless::String<32>,
    ip: (u8, u8, u8, u8)
}

#[repr(transparent)]
#[derive(Debug)]
#[derive(Clone)]
#[derive(derive_new::new)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
#[derive(bincode::Encode)]
#[derive(bincode::Decode)]
pub struct Record<T>(T);