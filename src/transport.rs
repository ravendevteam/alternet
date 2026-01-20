use std::{collections::HashMap, ops::DerefMut};

use crate::prelude::*;
use futures::TryFutureExt;
use libp2p::futures::StreamExt;

/// The possible errors of a [`Transport`] wrapped transport.
#[derive(Debug)]
pub enum Error<TErr> {
    /// The underlying transport encountered an error.
    Transport(TErr),
    /// Alternet DNS resolution failed.
    #[allow(clippy::enum_variant_names)]
    ResolveError(crate::control::resolve::Error),
    /// Alternet DNS resolution was successful, but the underlying transport refused the resolved address.
    MultiaddrNotSupported(multiaddr::Multiaddr),
    /// Multiple dial errors were encountered.
    Dial(Vec<Error<TErr>>),
    /// Alternet DNS registration failed
    RegisterError(crate::control::register::Error),
    /// Already listening on that domain name
    AlreadyListening(String),
}
impl<TErr> Error<TErr> {
    fn map_transport(
        transport_error: TransportError<InnerError>,
    ) -> TransportError<Error<InnerError>> {
        TransportError::Other(Error::<TErr>::map(transport_error))
    }
    fn map(transport_error: TransportError<InnerError>) -> Error<InnerError> {
        match transport_error {
            TransportError::MultiaddrNotSupported(multiaddr) => {
                Error::MultiaddrNotSupported(multiaddr)
            }
            TransportError::Other(inner_transport_error) => Error::Transport(inner_transport_error),
        }
    }
}
impl<TErr> std::fmt::Display for Error<TErr>
where
    TErr: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Transport(err) => write!(f, "{err}"),
            Error::ResolveError(err) => write!(f, "{err}"),
            Error::MultiaddrNotSupported(a) => write!(f, "Unsupported resolved address: {a}"),
            Error::Dial(errs) => {
                write!(f, "Multiple dial errors occurred:")?;
                for err in errs {
                    write!(f, "\n - {err}")?;
                }
                Ok(())
            }
            Error::RegisterError(err) => write!(f, "{err}"),
            Error::AlreadyListening(domain) => write!(f, "already listening on domain: {domain}"),
        }
    }
}
impl<TErr> std::error::Error for Error<TErr>
where
    TErr: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Transport(err) => Some(err),
            Error::ResolveError(err) => Some(err),
            Error::MultiaddrNotSupported(_) => None,
            Error::Dial(errs) => errs.last().and_then(|e| e.source()),
            Error::RegisterError(err) => Some(err),
            Error::AlreadyListening(_) => None,
        }
    }
}

// todo: wrong type. switch back to Boxed? probably
// pub type InnerTransport = core::transport::OrTransport<relay::client::Transport, T>;
pub type InnerTransport = core::transport::Boxed<(PeerId, core::muxing::StreamMuxerBox)>;
// pub type InnerTransport = core::transport::OrTransport<core::transport::Boxed<>, T>;
// pub type InnerTransport = relay::client::Transport;

struct DomainIds {
    id_to_domain: HashMap<core::transport::ListenerId, String>,
    domain_to_ids: HashMap<String, Vec<core::transport::ListenerId>>,
}
impl DomainIds {
    // returns true if it was a new domain
    fn add(&mut self, id: core::transport::ListenerId, domain: String) -> bool {
        let None = self.id_to_domain.insert(id, domain.clone()) else {
            panic!("we should not ever get here unless ids are reassigned")
        };
        let entry = self.domain_to_ids.entry(domain);
        let new_domain = matches!(entry, std::collections::hash_map::Entry::Vacant(_));
        let ids = entry.or_default();
        ids.push(id);
        return new_domain;
    }
    fn remove_id(&mut self, id: core::transport::ListenerId) -> Option<String> {
        if let Some(domain) = self.id_to_domain.remove(&id) {
            let Some(ids) = self.domain_to_ids.get_mut(&domain) else {
                unreachable!()
            };
            for (i, it) in ids.iter().enumerate() {
                if *it == id {
                    ids.remove(i);
                    break;
                }
            }
            Some(domain)
        } else {
            None
        }
    }
    // no situation where we have to remove a domain yet in library code for transport

    // fn remove_domain(&mut self, domain: &str) -> Option<Vec<core::transport::ListenerId>> {
    //     if let Some(ids) = self.domain_to_ids.remove(domain) {
    //         for id in ids.iter() {
    //             self.id_to_domain.remove(id);
    //         }
    //         Some(ids)
    //     } else {
    //         None
    //     }
    // }
}

// todo: either of:
// - make an inner transport that lives under only one mutex (preferred)
// - set an order in which all mutexes have to be acquired in and enforce this everywhere
// right now the code can probably deadlock somewhere

/// alternet::Transport acts as a wrapper around relay (actually OrTransport<relay::client::Transport, T>)
/// and allows you to use `/dns/yourdomain.an` anywhere you would normally use `/p2p/QmYourPeerId`
pub struct Transport {
    control: crate::control::Control,
    peerid: PeerId,
    inner: std::sync::Arc<parking_lot::Mutex<InnerTransport>>,
    // pending items:
    domains: parking_lot::Mutex<DomainIds>,
    // - dns registration
    // if we ever do deregistration, wrap this in `futures::future::Abortable` i guess
    registrations: futures::stream::FuturesUnordered<crate::control::register::RegisterFut>,
    // deregistrations: futures::stream::FuturesUnordered<
    //     crate::control::deregister::DeregisterFut,
    // >,
}

type InnerOutput = <InnerTransport as ::libp2p::Transport>::Output;
type InnerError = <InnerTransport as ::libp2p::Transport>::Error;
type InnerListenerUpgrade = <InnerTransport as ::libp2p::Transport>::ListenerUpgrade;
type InnerDial = <InnerTransport as ::libp2p::Transport>::Dial;

#[pin_project::pin_project]
pub struct Dial {
    control: crate::control::Control,
    inner: std::sync::Arc<parking_lot::Mutex<InnerTransport>>,
    opts: core::transport::DialOpts,

    current_inner_dial: Option<InnerDial>,
    dial_errs: Vec<Error<InnerError>>,

    // multiaddr dns-lookup / replacement part
    start: Multiaddr,
    replacements: Vec<(usize, Vec<Multiaddr>, Multiaddr)>,
    // latest resolved multiaddr
    current_replacement: Option<(Multiaddr, usize)>,
    resolves: stream::SelectAll<crate::control::resolve::ResolveFut>,
}

impl Future for Dial {
    type Output = Result<InnerOutput, Error<InnerError>>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.as_mut().project();

        if let Some(fut) = &mut this.current_inner_dial {
            match fut.poll_unpin(cx) {
                std::task::Poll::Ready(Ok(res)) => {
                    return std::task::Poll::Ready(Ok(res));
                }
                std::task::Poll::Ready(Err(e)) => {
                    // todo: map broken? some could be unsupported addr i guess?
                    // probably time to introduce a RelayError to Error
                    this.dial_errs.push(Error::Transport(e));
                }
                std::task::Poll::Pending => todo!(),
            }
        }

        'pending: loop {
            // if any /an/domain is still unresolved, poll next one
            for (i, addrs, _after) in this.replacements.iter() {
                if *i == addrs.len() {
                    break 'pending;
                }
            }

            let mut check_addr = this.start.clone();
            if let Some((current_addr, current_rep_i)) = this.current_replacement.take() {
                let mut next_replacement = true;
                for (rep_i, (i, addrs, after)) in this.replacements.iter_mut().enumerate() {
                    let addr = if rep_i == current_rep_i {
                        &current_addr
                    } else {
                        &addrs[*i]
                    };
                    for proto in addr.iter() {
                        check_addr.push(proto);
                    }
                    for proto in after.iter() {
                        check_addr.push(proto);
                    }
                    if rep_i == current_rep_i {
                        continue;
                    }
                    if *i < addrs.len() {
                        *i += 1;
                        next_replacement = false;
                        break;
                    }
                    *i = 0;
                }
                *this.current_replacement = if next_replacement {
                    let addrs = &mut this.replacements[current_rep_i].1;
                    addrs.push(current_addr);
                    None
                } else {
                    Some((current_addr, current_rep_i))
                }
            }
            // test
            let mut inner = this.inner.lock();
            match inner.dial(check_addr, *this.opts) {
                Ok(fut) => {
                    *this.current_inner_dial = Some(fut);
                    break 'pending;
                }
                Err(e) => {
                    let err: Error<InnerError> = Error::<InnerError>::map(e);
                    this.dial_errs.push(err);
                }
            }
        }

        if let Some((repl_addr, repl_i)) = this.current_replacement.take() {
            this.replacements[repl_i].1.push(repl_addr);
        }

        std::task::Poll::Pending
    }
}

impl ::libp2p::Transport for Transport {
    type Output = InnerOutput;
    type Error = Error<InnerError>;
    type ListenerUpgrade =
        futures::future::MapErr<InnerListenerUpgrade, fn(InnerError) -> Self::Error>;
    type Dial = Dial;
    // // either:
    // // - result of T::dial with its error mapped to Self::Error
    // // - (boxed async block) returning a Result-wrapped Self::Error
    // futures::future::Either<
    //     futures::future::MapErr<InnerDial, fn(InnerError) -> Self::Error>,
    //     futures::future::BoxFuture<'static, Result<Self::Output, Self::Error>>,
    // >;

    fn listen_on(
        &mut self,
        id: core::transport::ListenerId,
        mut addr: Multiaddr,
    ) -> Result<(), TransportError<Self::Error>> {
        // ~~todo: rewrite any `/an/domain`s before a /p2p-circuit so that we can use relays by alternet-domain~~
        // actually: the relay-behaviour does the subscribing on the relay_addr, so it will loop around to a dial later and be rewritten there

        fn is_alternet_dns_record(p: multiaddr::Protocol) -> bool {
            matches!(p, multiaddr::Protocol::Dns(_))
        }
        fn is_p2p_circuit(p: multiaddr::Protocol) -> bool {
            matches!(p, multiaddr::Protocol::P2pCircuit)
        }
        let p2p_pos = addr.iter().position(|p| is_p2p_circuit(p)).unwrap_or(0);
        let domain_pos = addr
            .iter()
            .skip(p2p_pos)
            .position(is_alternet_dns_record)
            .map(|x| x + p2p_pos);

        // if contains /an/domain in some reasonable position?
        // some reasonable position: anywhere not before a (/the last) /p2p-circuit
        if let Some(domain_pos) = domain_pos {
            // if there are multiple
            if addr
                .iter()
                .skip(domain_pos + 1)
                .position(|p| is_alternet_dns_record(p))
                .is_some()
            {
                return Err(TransportError::MultiaddrNotSupported(addr));
            }
            let Some(multiaddr::Protocol::Dns(domain)) = addr.iter().nth(domain_pos) else {
                unreachable!()
            };

            let domain: String = domain.to_string();
            let mut domains_lock = self.domains.lock();

            let is_new_domain = domains_lock.add(id, domain.clone());
            if is_new_domain {
                let register_fut = self.control.register(domain.clone());
                self.registrations.push(register_fut);
            }

            // rewrite multiaddr and replace /an/domain with /p2p/QmOurPeerId
            addr = addr
                .replace(domain_pos, |_| Some(multiaddr::Protocol::P2p(self.peerid)))
                .expect("i return a some, how would this ever fail?");
        }

        let mut inner_lock = self.inner.lock();
        inner_lock
            .listen_on(id, addr)
            .map_err(Error::<InnerError>::map_transport)
    }

    fn remove_listener(&mut self, id: core::transport::ListenerId) -> bool {
        // should remove_listener remove our dns entry from the network? -> no?
        {
            let mut domains_lock = self.domains.lock();
            if let Some(domain) = domains_lock.remove_id(id) {
                if domains_lock.domain_to_ids.contains_key(&domain) {
                    // if we want to add deregistring a domain the place would be here

                    // let deregistration = self.control.deregister(domain);
                    // self.deregistrations.push(deregistration);
                }
            }
        }
        let mut inner_lock = self.inner.lock();
        inner_lock.remove_listener(id)
    }

    fn dial(
        &mut self,
        addr: Multiaddr,
        opts: core::transport::DialOpts,
    ) -> Result<Self::Dial, TransportError<Self::Error>> {
        // this replaces a dns part of the multiaddr with our domain
        let mut iter = addr.iter();
        let mut start = Multiaddr::empty();

        let mut domains = vec![];
        while let Some(p) = iter.next() {
            if let multiaddr::Protocol::Dns(domain) = p {
                domains.push(domain.to_string());
                break;
            }
            start.push(iter.next().unwrap());
        }
        let mut replacements: Vec<(usize, Vec<Multiaddr>, Multiaddr)> = vec![];
        'done: loop {
            let mut addr = Multiaddr::empty();
            loop {
                if let Some(p) = iter.next() {
                    if let multiaddr::Protocol::Dns(domain) = p {
                        domains.push(domain.to_string());
                        replacements.push((0, vec![], addr));
                        break;
                    }
                    addr.push(iter.next().unwrap());
                } else {
                    break 'done;
                }
            }
        }
        let resolves: stream::SelectAll<_> = FromIterator::from_iter(
            domains
                .into_iter()
                .map(|domain| self.control.resolve(domain)),
        );
        Ok(Dial {
            control: self.control.clone(),
            inner: self.inner.clone(),
            opts,
            current_inner_dial: None,
            dial_errs: vec![],
            start,
            replacements,
            current_replacement: None,
            resolves,
        })
    }

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<core::transport::TransportEvent<Self::ListenerUpgrade, Self::Error>> {
        if let std::task::Poll::Ready(Some(res)) = self.registrations.poll_next_unpin(cx) {
            _ = res;
            // todo: tracing or smth
        }

        let mut inner_lock = self.inner.lock();

        // needs to be a fn pointer
        fn map_upgrade<F: TryFutureExt>(
            upgr: F,
        ) -> future::MapErr<F, fn(F::Error) -> Error<F::Error>> {
            upgr.map_err(Error::Transport)
        }

        libp2p::core::Transport::poll(std::pin::Pin::new(inner_lock.deref_mut()), cx)
            .map(|event| event.map_upgrade(map_upgrade).map_err(Error::Transport))
    }
}
