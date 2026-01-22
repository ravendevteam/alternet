//! # definitions
//! - ``: string, if item then key for dht
//! - ${var}: named variable
//! - ()?+*: like in regex
//! - public multiaddr:
//!     multiaddresses that are reachable by other nodes for dhts (public)
//!     - `/ip4/38.123.42.53/tcp/420`
//!     - `/ip6/2a05:3102:2346:200:2c95:b9e7:b158:8f30/udp/1010/quic`
//!
//! # [`record`]s
//! every record is signed and contains the public key it is signed with and
//! the signature calculated from the digest {key+fields+pubkey}
//!
//! - `root:${root}` -> public multiaddr
//!     claims a root domain
//!     - `${root}`.length > limit for public ip
//!     - rootaddr has to be part of the dht with peerid matching signature
//!     only root nodes that have the rootaddr in their routing table serve this value
//!     oldest prevails, does not expire
//!
//! - `lease:(${sub}.)*${domain}.${root}` -> peerid + timestamp
//!     leases `(${sub}.)*${domain}.${root}` to ${peerid} until ${timestamp}
//!     - timestamp cannot be more then 24 hours in the future
//!     - signature has to be from owner of the super domain:
//!         - `${domain}.${root}` has to be signed by `root:${root}` owner
//!
//! - `dns:(${sub}.)*${domain}.${root}` -> [multiaddr] ( )?
//!     states that domain is reachable at these addresses
//!     - addrs (value)
//!     - chain [lease] (optional)
//!
//! - `dns:peerid.p2p` -> [multiaddr]
//!     valid iff:
//!     - signed by peerid
//!

use crate::prelude::*;
use ::std::io;
use libp2p::bytes::Buf;
use oxicode::{
    Decode, Encode,
    config::{Configuration, LittleEndian, NoLimit, Varint},
    de::{BorrowDecode, BorrowDecoder},
    decode_from_slice,
    enc::Writer,
};
use std::io::Read;

trait ThisErrorDecode: Sized {
    fn decode<'a>(
        decoder: &mut oxicode::de::DecoderImpl<oxicode::de::SliceReader<'a>, OxicodeConfig>,
    ) -> Result<Self, Error>;
}

#[derive(Debug, oxicode::Encode)]
pub enum Record {
    Root(RootRecord),
    Lease(LeaseRecord),
    Addr(AddrRecord),
}

#[derive(Debug)]
pub enum Error {
    KeyMismatch,
    UnknownRecordType,
    UnexpectedRecordType,
    Key(hickory_resolver::proto::ProtoError),
    MultiAddr(multiaddr::Error),
    Oxicode(oxicode::Error),
    PeerId(identity::ParseError),
    PublicKey(identity::DecodingError),
    InvalidSignature,
    NoPublisher,
    WrongSigner,
    MoreData,
    NoExpiry,
    UnexpectedExpiry,
    Expired(std::time::Duration),
    TTLTooBig(std::time::Duration),
    // KeyDomainMismatch,
    // PublicKey(identity::DecodingError),
    // PublicKeyPublisherMismatch,
    // MultiAddr(multiaddr::Error),
    // InvalidSignature,
    // MoreBytes,
}

type OxicodeConfig = Configuration<LittleEndian, Varint, NoLimit>;
const REPUBLISH_INTERVAL: std::time::Duration = std::time::Duration::from_hours(24);
const EXPIRED_LEEWAY: std::time::Duration = std::time::Duration::from_secs(5);

// todo: to_kad_validate_chain

pub fn from_kad_validated(
    record: &kad::Record,
    now: std::time::Instant,
    now_system: std::time::SystemTime,
) -> Result<(Signed<Record>, Vec<Signed<LeaseRecord>>), Error> {
    if !record.value.starts_with(record.key.as_ref()) {
        return Err(Error::KeyMismatch);
    }
    if let Some(expiry) = record.expires {
        if let Some(future) = expiry.checked_duration_since(now + REPUBLISH_INTERVAL) {
            return Err(Error::TTLTooBig(future));
        }
    }
    let Some(orig_pub) = record.publisher else {
        // todo: does this ever organically happen?
        return Err(Error::NoPublisher);
    };
    let reader = oxicode::de::SliceReader::new(&record.value);
    let mut decoder = oxicode::de::DecoderImpl::new(reader, OxicodeConfig::default());
    let signed_record = <Signed<Record>>::decode(&mut decoder)?;
    if signed_record.pubkey.to_peer_id() != orig_pub {
        return Err(Error::WrongSigner);
    }
    let mut name_owned;
    let mut name = match &signed_record.signed {
        Record::Root(rootrecord) => {
            if record.expires.is_some() {
                // todo: really care about expires?
                return Err(Error::UnexpectedExpiry);
            }
            &rootrecord.root
        }
        Record::Lease(lease_record) => {
            // todo: ignore record.expires?
            if let Some(expiry) = record.expires {
                if let Some(expired) = now
                    .checked_duration_since(expiry)
                    .map(|expired| expired.checked_sub(EXPIRED_LEEWAY))
                    .flatten()
                {
                    return Err(Error::Expired(expired));
                }
            } else {
                // todo: this ever happen accidentally?
                return Err(Error::NoExpiry);
            }

            if let Ok(expired) = now_system.duration_since(lease_record.until) {
                return Err(Error::Expired(expired));
            }

            &lease_record.subdomain.base_name()
        }
        Record::Addr(addr_record) => {
            // todo: care about expires?
            &addr_record.domain
        },
    };
    let mut last_pub = orig_pub;
    let mut leases = vec![];
    loop {
        if name.is_root() {
            break;
        }
        let lease = <Signed<LeaseRecord>>::decode(&mut decoder)?;

        if lease.signed.leasee != last_pub {
            return Err(Error::WrongSigner);
        }

        last_pub = lease.pubkey.to_peer_id();
        leases.push(lease);
        name_owned = name.base_name();
        name = &name_owned;
    }

    if decoder.reader().remaining().is_empty() {
        Ok((signed_record, vec![]))
    } else {
        Err(Error::MoreData)
    }
}
fn check_lease_chain(mut super_pub: PeerId, mut name: hickory_resolver::Name, leases: &[LeaseRecord]) {}

impl oxicode::Encode for RootRecord {
    fn encode<E: oxicode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), oxicode::Error> {
        let root = format!("root:{}", self.root.to_ascii());
        root.encode(encoder)?;
        let addr: &[u8] = self.addr.as_ref();
        addr.encode(encoder)?;
        Ok(())
    }
}
impl oxicode::Encode for LeaseRecord {
    fn encode<E: oxicode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), oxicode::Error> {
        let subdomain = format!("lease:{}", self.subdomain.to_ascii());
        subdomain.encode(encoder)?;
        let leasee = self.leasee.to_bytes();
        leasee.encode(encoder)?;
        self.until.encode(encoder)?;
        Ok(())
    }
}
impl oxicode::Encode for AddrRecord {
    fn encode<E: oxicode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), oxicode::Error> {
        let domain = format!("lease:{}", self.domain.to_ascii());
        domain.encode(encoder)?;
        self.addrs.len().encode(encoder)?;
        for addr in self.addrs.iter() {
            let addr: &[u8] = addr.as_ref();
            addr.encode(encoder)?;
        }
        Ok(())
    }
}

impl ThisErrorDecode for RootRecord {
    fn decode<'a>(
        decoder: &mut oxicode::de::DecoderImpl<oxicode::de::SliceReader<'a>, OxicodeConfig>,
    ) -> Result<Self, Error> {
        let key = <&str>::borrow_decode(decoder).map_err(Error::Oxicode)?;
        let Some(root) = key.strip_prefix("root:") else {
            return Err(Error::UnexpectedRecordType);
        };

        let root = hickory_resolver::Name::from_ascii(root).map_err(Error::Key)?;
        // todo: root domain format

        let addr = <Vec<u8>>::decode(decoder).map_err(Error::Oxicode)?;
        let addr = Multiaddr::try_from(addr).map_err(Error::MultiAddr)?;

        Ok(RootRecord { root, addr })
    }
}
impl ThisErrorDecode for LeaseRecord {
    fn decode<'a>(
        decoder: &mut oxicode::de::DecoderImpl<oxicode::de::SliceReader<'a>, OxicodeConfig>,
    ) -> Result<Self, Error> {
        let key = <&str>::borrow_decode(decoder).map_err(Error::Oxicode)?;
        let Some(subdomain) = key.strip_prefix("lease:") else {
            return Err(Error::UnexpectedRecordType);
        };
        let subdomain = hickory_resolver::Name::from_ascii(subdomain).map_err(Error::Key)?;
        // todo: subdomain format?

        let leasee = <&[u8]>::borrow_decode(decoder).map_err(Error::Oxicode)?;
        let leasee = PeerId::from_bytes(leasee).map_err(Error::PeerId)?;
        let until = std::time::SystemTime::decode(decoder).map_err(Error::Oxicode)?;

        Ok(LeaseRecord {
            subdomain,
            leasee,
            until,
        })
    }
}
impl ThisErrorDecode for AddrRecord {
    fn decode<'a>(
        decoder: &mut oxicode::de::DecoderImpl<oxicode::de::SliceReader<'a>, OxicodeConfig>,
    ) -> Result<Self, Error> {
        let key = <&str>::borrow_decode(decoder).map_err(Error::Oxicode)?;
        let Some(domain) = key.strip_prefix("addr:") else {
            return Err(Error::UnexpectedRecordType);
        };
        let domain = hickory_resolver::Name::from_ascii(domain).map_err(Error::Key)?;
        // todo: domain format?

        let addrs_len = usize::decode(decoder).map_err(Error::Oxicode)?;
        let mut addrs = Vec::with_capacity(addrs_len);
        for _ in 0..addrs_len {
            let addr_vec = <Vec<u8>>::decode(decoder).map_err(Error::Oxicode)?;
            let addr = Multiaddr::try_from(addr_vec).map_err(Error::MultiAddr)?;
            addrs.push(addr);
        }

        Ok(AddrRecord { domain, addrs })
    }
}

impl ThisErrorDecode for Record {
    fn decode<'a>(
        decoder: &mut oxicode::de::DecoderImpl<oxicode::de::SliceReader<'a>, OxicodeConfig>,
    ) -> Result<Self, Error> {
        let rem = decoder.borrow_reader().remaining();
        let (key, _) = oxicode::borrow_decode_from_slice::<&[u8]>(rem).map_err(Error::Oxicode)?;
        if key.starts_with(b"root:") {
            RootRecord::decode(decoder).map(Record::Root)
        } else if key.starts_with(b"lease:") {
            LeaseRecord::decode(decoder).map(Record::Lease)
        } else if key.starts_with(b"addr:") {
            AddrRecord::decode(decoder).map(Record::Addr)
        } else {
            Err(Error::UnknownRecordType)
        }
    }
}

// impl TryFrom<(std::time::SystemTime, kad::Record)> for Record {
//     type Error = ();

//     fn try_from(value: kad::Record) -> Result<Self, Self::Error> {

//     }
// }
#[derive(Debug, Clone)]
pub struct RootRecord {
    pub root: hickory_resolver::Name,
    pub addr: Multiaddr,
}

#[derive(Debug, Clone)]
pub struct LeaseRecord {
    pub subdomain: hickory_resolver::Name,
    pub leasee: PeerId,
    pub until: std::time::SystemTime,
}
#[derive(Debug, Clone)]
pub struct AddrRecord {
    pub domain: hickory_resolver::Name,
    pub addrs: Vec<Multiaddr>,
}

#[derive(Debug, Clone)]
pub struct Signed<T> {
    signed: T,
    pubkey: identity::PublicKey,
    signature: Vec<u8>,
}
// impl<T> Into<T> for Signed<T> {
//     fn into(self) -> T {
//         self.signed
//     }
// }
// impl<T> From<Signed<T>> for T {
//     fn from(from: Signed<T>) -> T {
//         from.signed
//     }
// }
impl<T> AsRef<T> for Signed<T> {
    fn as_ref(&self) -> &T {
        &self.signed
    }
}

macro_rules! impl_signed_t {
    (Signed<$T:ident>) => {
        impl oxicode::Encode for Signed<$T> {
            fn encode<E: oxicode::enc::Encoder>(
                &self,
                encoder: &mut E,
            ) -> Result<(), oxicode::Error> {
                self.signed.encode(encoder)?;
                let pubkey = self.pubkey.encode_protobuf();
                pubkey.encode(encoder)?;
                self.signature.encode(encoder)?;

                Ok(())
            }
        }
    };
}

impl<T> Into<kad::Record> for Signed<T>
where
    Signed<T>: oxicode::Encode,
{
    fn into(self) -> kad::Record {
        let writer = oxicode::enc::VecWriter::new();
        let mut encoder = oxicode::enc::EncoderImpl::new(writer, OxicodeConfig::default());
        self.encode(&mut encoder).expect("VecWriter cannot fail");
        let value = encoder.into_writer().into_vec();
        let (key, _) =
            decode_from_slice::<Vec<u8>>(&value[..]).expect("i just wrote it, it can't fail");
        kad::Record::new(key, value)
    }
}

impl_signed_t!(Signed<RootRecord>);
impl_signed_t!(Signed<LeaseRecord>);
impl_signed_t!(Signed<AddrRecord>);
impl_signed_t!(Signed<Record>);

impl<T: AsRef<[u8]>> oxicode::Encode for Signed<T> {
    fn encode<E: oxicode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), oxicode::Error> {
        let encoded_bytes = self.signed.as_ref();
        encoder.writer().write(encoded_bytes)?;
        let pubkey = self.pubkey.encode_protobuf();
        pubkey.encode(encoder)?;
        self.signature.encode(encoder)?;

        Ok(())
    }
}

impl<T: ThisErrorDecode> ThisErrorDecode for Signed<T> {
    fn decode<'a>(
        decoder: &mut oxicode::de::DecoderImpl<oxicode::de::SliceReader<'a>, OxicodeConfig>,
    ) -> Result<Self, Error> {
        let rem = decoder.reader().remaining();
        let signed = T::decode(decoder)?;
        let signed_bytes = &rem[0..rem.len() - decoder.reader().remaining().len()];

        let pubkey = <&[u8]>::borrow_decode(decoder).map_err(Error::Oxicode)?;
        let pubkey = identity::PublicKey::try_decode_protobuf(pubkey).map_err(Error::PublicKey)?;
        let signature = <Vec<u8>>::decode(decoder).map_err(Error::Oxicode)?;

        if !pubkey.verify(signed_bytes, &signature) {
            return Err(Error::InvalidSignature);
        }

        Ok(Signed {
            signed,
            pubkey,
            signature,
        })
    }
}

impl AddrRecord {
    // /// all sizes/lens encoded in big endian
    // /// encoded in order of appearance:
    // /// [ domain_len (u8) | domain ]
    // /// [ addrs_len (u8) |
    // ///     [ addr_len (u16) | addr.. ]
    // /// ]
    // /// [ pubkey_len (u16) | [pubkey][pubkey-peerid-spec] ]
    // /// [ signature_len (u16) | signature.. ]
    // ///
    // /// # Panics
    // ///
    // /// - if pubkey is >= 2^16 bytes long
    // /// - if there are >= 2^16 addrs
    // ///
    // /// [pubkey-peerid-spec]: https://github.com/libp2p/specs/blob/e87cb1c32a666c2229d3b9bb8f9ce1d9cfdaa8a9/peer-ids/peer-ids.md
    // fn to_buf(&self, w: &mut impl io::Write) -> io::Result<()> {
    //     let domain_bytes = &self.domain.0[..];
    //     let domain_len = [u8::try_from(domain_bytes.len()).expect("malformed domain??")];
    //     let addrs_len = u16::to_be_bytes(u16::try_from(self.addrs.len()).expect("that is too many addrs"));

    //     let mut ioslices = [
    //         &domain_len[..],
    //         domain_bytes,
    //         &addrs_len[..]
    //     ].map(io::IoSlice::new);
    //     w.write_all_vectored(&mut ioslices[..])

    //     let pubkey_bytes = key.public().encode_protobuf();
    //     let pubkey_len = u16::to_be_bytes(u16::try_from(pubkey_bytes.len()).expect("wow that is quite long"));

    // }
}

// pub enum FromKadError2 {
//     Key(DomainError),
//     NoPublisher,
//     Format(io::Error),
//     KeyDomainMismatch,
//     PublicKey(identity::DecodingError),
//     PublicKeyPublisherMismatch,
//     MultiAddr(multiaddr::Error),
//     InvalidSignature,
//     MoreBytes,
// }

// impl TryFrom<kad::Record> for AddrRecord {
//     type Error = FromKadError2;
//     fn try_from(kad: kad::Record) -> Result<Self, Self::Error> {
//         let key_domain = match check_domain_name(kad.key.to_vec()) {
//             Ok(domain) => domain,
//             Err(e) => return Err(FromKadError2::Key(e)),
//         };
//         let Some(publisher) = kad.publisher else {
//             return Err(FromKadError2::NoPublisher);
//         };
//         let mut cursor = io::Cursor::new(&kad.value[..]);

//         let mut domain_len_u8_buf = [0u8; 1];
//         cursor
//             .read_exact(&mut domain_len_u8_buf)
//             .map_err(FromKadError2::Format)?;
//         let domain_len = domain_len_u8_buf[0];
//         let mut domain = vec![0u8; domain_len as usize];
//         cursor
//             .read_exact(&mut domain[0..domain_len as usize])
//             .map_err(FromKadError2::Format)?;

//         if domain != kad.key.as_ref() {
//             return Err(FromKadError2::KeyDomainMismatch);
//         }

//         let mut addrs_len_u8_buf = [0u8; 1];
//         cursor
//             .read_exact(&mut addrs_len_u8_buf)
//             .map_err(FromKadError2::Format)?;
//         let addrs_len = addrs_len_u8_buf[0];
//         let mut addrs = Vec::with_capacity(addrs_len as usize);
//         for _ in 0..addrs_len {
//             let mut addr_len_u16_buf = [0; std::mem::size_of::<u16>()];
//             cursor
//                 .read_exact(&mut addr_len_u16_buf)
//                 .map_err(FromKadError2::Format)?;
//             let addr_len = u16::from_be_bytes(addr_len_u16_buf);
//             let mut addr_bytes = vec![0; addr_len as usize];
//             cursor
//                 .read_exact(&mut addr_bytes)
//                 .map_err(FromKadError2::Format)?;
//             let addr =
//                 multiaddr::Multiaddr::try_from(addr_bytes).map_err(FromKadError2::MultiAddr)?;
//             addrs.push(addr);
//         }

//         let mut pubkey_len_u16_buf = [0u8; std::mem::size_of::<u16>()];
//         cursor
//             .read_exact(&mut pubkey_len_u16_buf)
//             .map_err(FromKadError2::Format)?;
//         let pubkey_len = u16::from_be_bytes(pubkey_len_u16_buf);
//         let mut pubkey_bytes = vec![0u8; pubkey_len as usize];
//         cursor
//             .read_exact(&mut pubkey_bytes)
//             .map_err(FromKadError2::Format)?;
//         let pubkey = identity::PublicKey::try_decode_protobuf(&pubkey_bytes)
//             .map_err(FromKadError2::PublicKey)?;

//         if pubkey.to_peer_id() != publisher {
//             return Err(FromKadError2::PublicKeyPublisherMismatch);
//         }

//         let signed_data = &kad.value[0..cursor.position() as usize];

//         let mut signature_len_u16_buf = [0; std::mem::size_of::<u16>()];
//         cursor
//             .read_exact(&mut signature_len_u16_buf)
//             .map_err(FromKadError2::Format)?;
//         let signature_len = u16::from_be_bytes(signature_len_u16_buf);
//         let mut signature = vec![0; signature_len as usize];
//         cursor
//             .read_exact(&mut signature)
//             .map_err(FromKadError2::Format)?;

//         if cursor.has_remaining() {
//             return Err(FromKadError2::MoreBytes);
//         }

//         if !pubkey.verify(signed_data, &signature) {
//             return Err(FromKadError2::InvalidSignature);
//         }

//         Ok(Self {
//             domain: todo!(),
//             // pubkey,
//             addrs,
//             // signature,
//         })
//     }
// }

// pub struct Domain(Vec<u8>);

// enum DomainError {
//     TooLong,
//     LabelEmpty,
//     LabelTooLong,
//     LabelStart,
//     LabelMiddle,
//     LabelEnd,
//     NotAlternetTLD,
// }

// fn make_user_domain(domain: impl Into<Vec<u8>>) -> Result<Domain, DomainError> {
//     let mut vec: Vec<u8> = domain.into();
//     vec.make_ascii_uppercase();
//     check_domain_name(vec)
// }

// // domain but all uppercase
// // https://www.rfc-editor.org/rfc/rfc1035
// fn check_domain_name(domain: Vec<u8>) -> Result<Domain, DomainError> {
//     // https://www.rfc-editor.org/rfc/rfc1035#section-2.3.4
//     if domain.len() > 255 {
//         return Err(DomainError::TooLong);
//     };

//     let labels = domain.split(|c| *c == b'.');
//     for label in labels {
//         // https://www.rfc-editor.org/rfc/rfc1035#section-2.3.4
//         if label.len() == 0 {
//             return Err(DomainError::LabelEmpty);
//         }
//         if label.len() > 63 {
//             return Err(DomainError::LabelTooLong);
//         }

//         // https://www.rfc-editor.org/rfc/rfc1035#section-2.3.1
//         // start has to be letter
//         if !label[0].is_ascii_uppercase() {
//             return Err(DomainError::LabelStart);
//         }
//         // end has to be letter or digit
//         if !label[label.len() - 1].is_ascii_uppercase() && !label[0].is_ascii_digit() {
//             return Err(DomainError::LabelEnd);
//         }
//         // every (other but who cares) character has to be
//         for c in label {
//             if !c.is_ascii_uppercase() && !c.is_ascii_digit() && *c != b'-' {
//                 return Err(DomainError::LabelMiddle);
//             }
//         }
//     }

//     // i guess we doin this
//     if domain.ends_with(b".AN") {
//         Ok(Domain(domain))
//     } else {
//         Err(DomainError::NotAlternetTLD)
//     }
// }
