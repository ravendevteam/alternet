use super::*;

pub struct Version {
    major: u32,
    minor: u32,
    patch: u32
}

impl Version {
    pub fn from_major(major: u32) -> Self {
        let minor: u32 = 0;
        let patch: u32 = 0;
        (major, minor, patch).into()
    }

    pub fn from_minor(minor: u32) -> Self {
        let major: u32 = 0;
        let patch: u32 = 0;
        (major, minor, patch).into()
    }

    pub fn from_patch(patch: u32) -> Self {
        let major: u32 = 0;
        let minor: u32 = 0;
        (major, minor, patch).into()
    }
}

impl Version {
    pub fn major(&self) -> u32 {
        self.major
    }

    pub fn minor(&self) -> u32 {
        self.minor
    }

    pub fn patch(&self) -> u32 {
        self.patch
    }

    pub fn increment_major(&mut self) {
        self.major += 1;
    }

    pub fn increment_minor(&mut self) {
        self.minor += 1;
    }

    pub fn increment_patch(&mut self) {
        self.patch += 1;
    }
}

impl<A, B, C> From<(A, B, C)> for Version 
where
    A: Into<u32>,
    B: Into<u32>,
    C: Into<u32> {
    fn from(value: (A, B, C)) -> Self {
        let major: u32 = value.0.into();
        let minor: u32 = value.1.into();
        let patch: u32 = value.2.into();
        Self {
            major,
            minor,
            patch
        }
    }
}

impl<A, B> From<(A, B)> for Version
where
    A: Into<u32>,
    B: Into<u32> {
    fn from(value: (A, B)) -> Self {
        let major: u32 = value.0.into();
        let minor: u32 = value.1.into();
        let patch: u32 = 0;
        (major, minor, patch).into()
    }
}

impl<A, B, C> std::ops::Add<(A, B, C)> for Version 
where
    A: Into<u32>,
    B: Into<u32>,
    C: Into<u32> {
    type Output = Self;

    fn add(self, rhs: (A, B, C)) -> Self::Output {
        let major: u32 = rhs.0.into();
        let major: u32 = self.major + major;
        let minor: u32 = rhs.1.into();
        let minor: u32 = self.minor + minor;
        let patch: u32 = rhs.2.into();
        let patch: u32 = self.patch + patch;
        (major, minor, patch).into()
    }
}

impl<A, B, C> std::ops::AddAssign<(A, B, C)> for Version
where
    A: Into<u32>,
    B: Into<u32>,
    C: Into<u32> {
    fn add_assign(&mut self, rhs: (A, B, C)) {
        let major: u32 = rhs.0.into();
        let minor: u32 = rhs.1.into();
        let patch: u32 = rhs.2.into();
        self.major + major;
        self.minor + minor;
        self.patch + patch;
    }
}




fn t() {
    let major: u32 = 0;
    let minor: u32 = 1;
    let patch: u32 = 0;
    let mut version: Version = (major, minor, patch).into();
    let major: u32 = 1;
    let minor: u32 = 0;
    let patch: u32 = 0;
    version += (major, minor, patch);
}






// Topic-Based Naming Structure

// what are the rules?


// # Topics
// 
// 5


// are topic fixed??
// copyright infringement??
// are domains known at compile time??
// do new topic need to be registered?? by who?? can anyone create a topic??

// is jinusb9ubss a valid topic??

// max topic length??

// can use request_response behaviour to add this as its own protocol
// ```rs
// #[derive(Debug)]
// pub struct Request {
//    pub url: Url,
//    pub payload: Option<String>,
// }
//
// #[derive(Debug)]
// pub struct Response {
//    pub status: u16,
//    pub payload: Option<String>,
// }
// ```

// do we need a topic registry stored on the kad too??

// an://shark.*


// needs to support sub domains
pub struct Url(String);

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "an://{}:{}", self.domain, self.topic)
    }
}







pub struct Signature(String);

#[derive(Debug)]
#[derive(Clone)]
#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
#[derive(bincode::Encode)]
#[derive(bincode::Decode)]
pub struct Ttl {
    pub times_republished: u32,
    pub timestamp_republished: std::time::Duration,
    pub time_to_live: std::time::Duration
}


pub struct Root {
    did: did::Did,
    look_ip: String,
    ttl: Ttl,
    version: Version
}



pub struct Metadata {
    pub name: String,
    pub log_uri: String,
    pub uri: String
}






// /oidc-auth-request/1.0
// Allows an external service, to initiate a "reverse-oidc-like" flow by requesting user consent for scoped access to the pod. Direction: Service → User Client (request), User Client → Service (response)


pub enum Scope {
    ReadProfile,
    WriteEvent
}

#[derive(Default)]
pub enum Locale {
    #[default]
    En,
    Fr
}




#[repr(transparent)]
pub struct Locales(Vec<Locale>);

impl Locales {
    pub fn add_locale(mut self, locale: Locale) -> Self {
        self.0.push(locale);
        self
    }
}




pub struct R {
    pub id: String,
    pub metadata: Metadata,
    pub scopes: Vec<Scope>,
    pub redirect_url: String,
    pub locales: Locales
}



// we can wrap kad
pub trait KadExt {
    fn put(&mut self, record: Record);
    fn get(&self) -> Record;
}






fn test2() {
    let x: Record = Record {
        name: "".to_owned()
    };

    let out = bincode::encode_to_vec(x, bincode::config::standard()).unwrap();
}


impl Record {
    pub fn to_kad_record(self) {
        let k: kad::RecordKey = kad::RecordKey::new(b"H");
        kad::Record::new(k, bincode::encode_to_vec(self, bincode::config::standard()).unwrap());
    }
}





#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
pub struct QueryParam {
    pub key: String,
    pub val: String
}


// zero sized?? blazingly fast? but more complicated?? do we need that speed

pub struct Get;
pub struct Post;
pub struct Put;
pub struct Delete;
pub struct Json;
pub struct Empty;

#[derive(serde::Serialize)]
#[derive(serde::Deserialize)]
#[serde(bound(serialize = "C: serde::Serialize"))]
#[serde(bound(deserialize = "C: serde::de::DeserializeOwned"))]
pub struct Request<A, B, C> {
    path: libp2p::multiaddr::Multiaddr,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    body: Option<C>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    query: Option<Vec<QueryParam>>,

    #[serde(skip)]
    phantom_data: std::marker::PhantomData<(A, B)>
}

pub trait ContentType {
    const VAL: &'static str;
}

impl ContentType for Json {
    const VAL: &'static str = "application/json";
}

impl ContentType for Empty {
    const VAL: &'static str = "";
}

pub trait Method {
    const NAME: &'static str;
}

impl Method for Get {
    const NAME: &'static str = "GET";
}


impl<A, B> Request<A, B, ()> 
where
    A: Method,
    B: ContentType {

}



pub struct PodAccessRequest<T> {
    pub method: Method,
    pub content_type: Option<String>,
    pub body: T,
}

pub struct Response<T> {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: T
}