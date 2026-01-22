use crate::prelude::*;

pub struct WithFallback<R> {
    resolver: crate::control::Control,
    fallback: R,
}

#[async_trait::async_trait]
impl<R: dns::Resolver + Sync> dns::Resolver for WithFallback<R> {
    async fn lookup_ip(
        &self,
        name: String,
    ) -> Result<hickory_resolver::lookup_ip::LookupIp, hickory_resolver::ResolveError> {
        self.fallback.lookup_ip(name).await
    }
    async fn ipv4_lookup(
        &self,
        name: String,
    ) -> Result<hickory_resolver::lookup::Ipv4Lookup, hickory_resolver::ResolveError> {
        self.fallback.ipv4_lookup(name).await
    }
    async fn ipv6_lookup(
        &self,
        name: String,
    ) -> Result<hickory_resolver::lookup::Ipv6Lookup, hickory_resolver::ResolveError> {
        self.fallback.ipv6_lookup(name).await
    }
    async fn txt_lookup(
        &self,
        name: String,
    ) -> Result<hickory_resolver::lookup::TxtLookup, hickory_resolver::ResolveError> {
        match self.resolver.txt_lookup(name.clone()).await {
            Ok(ok) => Ok(ok),
            Err(e) => {
                if let hickory_resolver::ResolveErrorKind::Proto(proto) = e.kind() {
                    if proto.kind().is_no_error() {
                        return self.fallback.txt_lookup(name).await;
                    }
                }
                Err(e)
            }
        }
    }
}

#[async_trait::async_trait]
impl libp2p::dns::Resolver for crate::control::Control {
    async fn lookup_ip(
        &self,
        _: String,
    ) -> Result<hickory_resolver::lookup_ip::LookupIp, hickory_resolver::ResolveError> {
        Err(hickory_resolver::proto::ProtoError::from(
            hickory_resolver::proto::ProtoErrorKind::NoError,
        )
        .into())
    }
    async fn ipv4_lookup(
        &self,
        _: String,
    ) -> Result<hickory_resolver::lookup::Ipv4Lookup, hickory_resolver::ResolveError> {
        Err(hickory_resolver::proto::ProtoError::from(
            hickory_resolver::proto::ProtoErrorKind::NoError,
        )
        .into())
    }
    async fn ipv6_lookup(
        &self,
        _: String,
    ) -> Result<hickory_resolver::lookup::Ipv6Lookup, hickory_resolver::ResolveError> {
        Err(hickory_resolver::proto::ProtoError::from(
            hickory_resolver::proto::ProtoErrorKind::NoError,
        )
        .into())
    }
    async fn txt_lookup(
        &self,
        name: String,
    ) -> Result<hickory_resolver::lookup::TxtLookup, hickory_resolver::ResolveError> {
        let name = into_alternet_name(name)?;

        let addrs = match self.resolve(name.clone()).await {
            Ok(Ok(addrs)) => addrs,
            Ok(Err(e)) => return Err(hickory_resolver::ResolveError::from(e)),
            Err(e) => {
                let proto_err_kind = hickory_resolver::proto::ProtoErrorKind::Canceled(e);
                return Err(hickory_resolver::proto::ProtoError::from(proto_err_kind).into());
            }
        };

        Ok(into_fake_txt_record(name, addrs))
    }
}

fn into_alternet_name(
    mut name: String,
) -> Result<hickory_resolver::Name, hickory_resolver::proto::ProtoError> {
    use hickory_resolver::*;

    if name.ends_with(".an") {
        name.pop();
        name.pop();
        name.pop();
        return Name::from_utf8(name);
    }
    if name.ends_with(".p2p") {
        name.pop();
        name.pop();
        name.pop();
        name.pop();
        return Name::from_utf8(name);
    }

    Err(proto::ProtoError::from(proto::ProtoErrorKind::NoError).into())
}

fn into_fake_txt_record(
    name: hickory_resolver::Name,
    addrs: Vec<Multiaddr>,
) -> hickory_resolver::lookup::TxtLookup {
    const DNS_CLASS: hickory_resolver::proto::rr::DNSClass =
        hickory_resolver::proto::rr::DNSClass::Unknown(u16::from_be_bytes(*b"AN"));
    const MAX_TTL: u32 = 86400u32;

    let mut query = hickory_resolver::proto::op::Query::new();
    query.set_query_class(DNS_CLASS);
    query.set_name(name.clone());
    query.set_query_type(hickory_resolver::proto::rr::RecordType::TXT);
    let records = addrs
        .into_iter()
        .map(|addr| vec![format!("dnsaddr={}", addr.to_string())])
        .map(hickory_resolver::proto::rr::rdata::TXT::new)
        .map(hickory_resolver::proto::rr::RData::TXT)
        .map(move |rdata| {
            hickory_resolver::proto::rr::Record::from_rdata(name.clone(), MAX_TTL, rdata)
        })
        .map(|mut record| {
            record.set_dns_class(DNS_CLASS);
            record
        })
        .collect::<Vec<_>>();

    hickory_resolver::lookup::Lookup::new_with_max_ttl(query, records.into()).into()
}
