use std::ops::DerefMut;

use ::libp2p::core::transport::DialOpts;
use ::libp2p::multiaddr::Protocol;
use ::libp2p::swarm::derive_prelude::{ConnectionHandler, Either};
use ::libp2p::swarm::{NetworkBehaviour, behaviour::toggle::Toggle, dummy};
use ::libp2p::{Multiaddr, Transport};
use futures_util::{FutureExt, TryFutureExt};
use tokio::sync::mpsc::UnboundedSender;

pub trait WithAlternetExt: Sized {
    fn with_an(&self, domain: &str) -> Self;
}
impl WithAlternetExt for Multiaddr {
    fn with_an(&self, domain: &str) -> Self {
        let mut buf = self.to_vec();
        let mut id_buf = unsigned_varint::encode::u32_buffer();
        buf.extend_from_slice(unsigned_varint::encode::u32(MULTIADDR_NUM, &mut id_buf));
        let mut strlen_buf = unsigned_varint::encode::usize_buffer();
        buf.extend_from_slice(unsigned_varint::encode::usize(
            domain.as_bytes().len(),
            &mut strlen_buf,
        ));
        buf.extend_from_slice(domain.as_bytes());

        // i guess that's how we roll now
        // yes, this is UB. no, i don't care
        return unsafe { std::mem::transmute(std::sync::Arc::new(buf)) };
    }
}

pub const MULTIADDR_NUM: u32 = 53; // (mis)use DNS

// if we can make sure that things exist only exactly once then this can also be
// [`tokio::sync::oneshot::Sender`]
pub type Sender<T> = tokio::sync::mpsc::UnboundedSender<T>;

pub struct Resolvable {
    pub domain: String,
    pub result_sender: Sender<Result<libp2p::PeerId, libp2p::kad::GetRecordError>>,
}

/// todo: add actual info
#[derive(Debug)]
pub struct ResolveError;
impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("error resolving Alternet Domain")
    }
}
impl std::error::Error for ResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// The possible errors of a [`Transport`] wrapped transport.
#[derive(Debug)]
pub enum Error<TErr> {
    /// The underlying transport encountered an error.
    Transport(TErr),
    /// Alternet DNS resolution failed.
    #[allow(clippy::enum_variant_names)]
    ResolveError(ResolveError),
    /// Alternet DNS resolution was successful, but the underlying transport refused the resolved address.
    MultiaddrNotSupported(Multiaddr),
    /// Multiple dial errors were encountered.
    Dial(Vec<Error<TErr>>),
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
        }
    }
}

pub struct AlternetBehaviour {
    request_receiver: tokio::sync::mpsc::Receiver<Resolvable>,
    request_sender: tokio::sync::mpsc::Sender<Resolvable>,
    lookups: std::sync::Arc<
        parking_lot::Mutex<
            std::collections::HashMap<
                libp2p::kad::QueryId,
                UnboundedSender<Result<libp2p::PeerId, libp2p::kad::GetRecordError>>,
            >,
        >,
    >,
    delegating: DelegatingBehaviour,
}

#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct DelegatingBehaviour {
    identify: libp2p::identify::Behaviour,
    kad: libp2p::kad::Behaviour<libp2p::kad::store::MemoryStore>,
    relay: Toggle<libp2p::relay::client::Behaviour>,
    // mdns: Toggle<libp2p::mdns::tokio::Behaviour>,
    // ping: Toggle<libp2p::ping::Behaviour>,
}

impl AlternetBehaviour {
    // todo: do we need this &mut? (probably because of shit like subdomainlookup)
    fn validate(&self, record: &libp2p::kad::Record) -> bool {
        todo!()
    }
}

impl NetworkBehaviour for AlternetBehaviour {
    type ConnectionHandler = libp2p::swarm::ConnectionHandlerSelect<
        dummy::ConnectionHandler,
        <DelegatingBehaviour as NetworkBehaviour>::ConnectionHandler,
    >;
    type ToSwarm = Either<(), <DelegatingBehaviour as NetworkBehaviour>::ToSwarm>;
    fn handle_established_inbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        Ok(ConnectionHandler::select(
            dummy::ConnectionHandler,
            self.delegating.handle_established_inbound_connection(
                connection_id,
                peer,
                local_addr,
                remote_addr,
            )?,
        ))
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: libp2p::PeerId,
        addr: &Multiaddr,
        role_override: libp2p::core::Endpoint,
        port_use: libp2p::core::transport::PortUse,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        Ok(ConnectionHandler::select(
            dummy::ConnectionHandler,
            self.delegating.handle_established_outbound_connection(
                connection_id,
                peer,
                addr,
                role_override,
                port_use,
            )?,
        ))
    }

    fn on_swarm_event(&mut self, event: libp2p::swarm::FromSwarm) {
        self.delegating.on_swarm_event(event);
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: libp2p::PeerId,
        connection_id: libp2p::swarm::ConnectionId,
        event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        use Either::*;
        let event = match event {
            Left(a) => match a {},
            Right(event) => event,
        };
        self.delegating
            .on_connection_handler_event(peer_id, connection_id, event);
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<libp2p::swarm::ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>>
    {
        use std::task::Poll;
        match self.delegating.poll(cx) {
            Poll::Ready(e) => {
                if let libp2p::swarm::ToSwarm::GenerateEvent(DelegatingBehaviourEvent::Kad(
                    libp2p::kad::Event::OutboundQueryProgressed {
                        id,
                        result: libp2p::kad::QueryResult::GetRecord(get_record_ok),
                        stats: _,
                        step,
                    },
                )) = &e
                {
                    let mut lookuplock = self.lookups.lock();
                    let maybe_callback = if step.last {
                        // remove if this is the last event as to not leak memory
                        lookuplock.remove(id).map(std::borrow::Cow::Owned)
                    } else {
                        lookuplock.get(id).map(std::borrow::Cow::Borrowed)
                    };
                    if let Some(sender) = maybe_callback {
                        match get_record_ok {
                            Ok(libp2p::kad::GetRecordOk::FoundRecord(
                                libp2p::kad::PeerRecord { record, peer },
                            )) => {
                                if self.validate(&record)
                                    && let Some(publisher) = record.publisher
                                {
                                    let _ = sender.send(Ok(publisher));
                                } else {
                                    // todo: --rep for the peer that sent us this bs
                                    _ = peer;
                                }
                            }
                            Err(e) => {
                                let _ = sender.send(Err(e.clone()));
                            }
                            _ => {}
                        }
                    }
                }

                return Poll::Ready(e.map_out(Either::Right).map_in(Either::Right));
            }
            Poll::Pending => {}
        }
        match self.request_receiver.poll_recv(cx) {
            Poll::Ready(e) => {
                if let Some(resolvable) = e {
                    let query_id = self
                        .delegating
                        .kad
                        .get_record(libp2p::kad::RecordKey::new(&resolvable.domain));
                    _ = self
                        .lookups
                        .lock()
                        .insert(query_id, resolvable.result_sender);
                } else {
                    unreachable!("this shouldn't happen") // probably shouldn't crash but let's see if this ever happens
                }
            }
            Poll::Pending => {}
        }
        Poll::Pending
    }
}

pub struct AlternetOptions {
    // pub dns: Option<hickory_resolver::config::ResolverConfig>,
    pub relay: bool,
}
impl Default for AlternetOptions {
    fn default() -> Self {
        Self {
            // dns: None,
            relay: false,
        }
    }
}

pub fn new<T>(
    keypair: &libp2p::identity::Keypair,
    wrapped_transport: T,
    opts: AlternetOptions,
) -> (AlternetTransport, AlternetBehaviour)
where
    T: libp2p::Transport<Output = (libp2p::PeerId, libp2p::core::muxing::StreamMuxerBox)>
        + Send
        + Unpin
        + 'static,
    <T as libp2p::Transport>::Error: 'static + Send + Sync,
    <T as libp2p::Transport>::Dial: Send + 'static,
    <T as libp2p::Transport>::ListenerUpgrade: Send,
    <T as libp2p::Transport>::ListenerUpgrade: 'static,
{
    let (sender, receiver) = tokio::sync::mpsc::channel(30);

    let local_id = keypair.public().to_peer_id();
    let identify_cfg =
        libp2p::identify::Config::new_with_signed_peer_record("/an/0.0.1".to_owned(), keypair);
    let kad_cfg = libp2p::kad::Config::new(libp2p::StreamProtocol::new("/an-kad/0.0.1"));

    let (relay_transport, relay_behaviour) = if opts.relay {
        let (relay_transport, relay_behaviour) = libp2p::relay::client::new(local_id);
        let relay_transport = relay_transport
            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
            .authenticate(
                libp2p::noise::Config::new(&keypair)
                    .expect("given a keypair this should work, right?"),
            )
            .multiplex(libp2p::yamux::Config::default())
            .map(|(p, c), _| (p, libp2p::core::muxing::StreamMuxerBox::new(c)));

        let relay_transport = libp2p::core::transport::OptionalTransport::some(relay_transport);

        (relay_transport, Some(relay_behaviour))
    } else {
        let relay_transport = libp2p::core::transport::OptionalTransport::none();

        (relay_transport, None)
    };

    let transport = AlternetTransport {
        inner: std::sync::Arc::new(parking_lot::Mutex::new(
            relay_transport
                .or_transport(wrapped_transport)
                .map(|either, _| either.into_inner())
                .boxed(),
        )),
        resolver: sender.clone(),
    };

    let delegating = DelegatingBehaviour {
        identify: libp2p::identify::Behaviour::new(identify_cfg),
        kad: libp2p::kad::Behaviour::with_config(
            local_id,
            libp2p::kad::store::MemoryStore::new(local_id),
            kad_cfg,
        ),
        relay: Toggle::from(relay_behaviour),
    };

    let behaviour = AlternetBehaviour {
        request_receiver: receiver,
        request_sender: sender,
        lookups: Default::default(),
        delegating,
    };

    (transport, behaviour)
}

pub type BoxedTransport =
    libp2p::core::transport::Boxed<(libp2p::PeerId, libp2p::core::muxing::StreamMuxerBox)>;

pub struct AlternetTransport {
    /// The underlying transport.
    inner: std::sync::Arc<parking_lot::Mutex<BoxedTransport>>,
    resolver: tokio::sync::mpsc::Sender<Resolvable>,
}

impl AlternetTransport {
    fn do_dial(
        &mut self,
        addr: Multiaddr,
        dial_opts: DialOpts,
    ) -> <Self as libp2p::Transport>::Dial {
        let resolver = self.resolver.clone();
        let inner = self.inner.clone();
        async move {
            let mut dial_errors: Vec<Error<<BoxedTransport as Transport>::Error>> = Vec::new();
            // let mut dns_lookups = 0;
            // let mut dial_attempts = 0;
            let mut addr = addr;

            if let Ok((Some(domain), extracted_addr)) = extract(addr.clone()) {
                let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
                resolver
                    .send(Resolvable {
                        domain,
                        result_sender: sender,
                    })
                    .await
                    .expect("resolver receiver dropped");

                while let Some(Ok(resolved)) = receiver.recv().await {
                    addr = extracted_addr.insert_p2p(resolved);
                }
            }
            let dial = inner.lock().dial(addr, dial_opts);
            let result = match dial {
                Ok(out) => {
                    // dial_attempts += 1;
                    out.await.map_err(Error::Transport)
                }
                Err(libp2p::TransportError::MultiaddrNotSupported(a)) => {
                    Err(Error::MultiaddrNotSupported(a))
                }
                Err(libp2p::TransportError::Other(err)) => Err(Error::Transport(err)),
            };

            match result {
                Ok(out) => Ok(out),
                Err(err) => {
                    dial_errors.push(err);

                    Err(Error::Dial(dial_errors))
                }
            }
        }
        .boxed()
        .right_future()
    }
}

struct AmendableMultiAddr {
    beginning: Vec<u8>,
    end: Vec<u8>,
}
impl AmendableMultiAddr {
    fn insert_p2p(&self, id: libp2p::PeerId) -> Multiaddr {
        let mut buf = self.beginning.clone();
        let mut id_buf = unsigned_varint::encode::u32_buffer();
        buf.extend_from_slice(unsigned_varint::encode::u32(421, &mut id_buf));
        let id_buf = id.to_bytes();
        let mut strlen_buf = unsigned_varint::encode::usize_buffer();
        buf.extend_from_slice(unsigned_varint::encode::usize(
            id_buf.len(),
            &mut strlen_buf,
        ));
        buf.extend_from_slice(&id_buf);
        buf.extend_from_slice(&self.end);

        // i guess that's how we roll now
        // yes, this is UB. no, i don't care
        return unsafe { std::mem::transmute(std::sync::Arc::new(buf)) };
    }
}

fn extract(
    addr: Multiaddr,
) -> Result<(Option<String>, AmendableMultiAddr), libp2p::multiaddr::Error> {
    fn split_at(n: usize, input: &[u8]) -> Result<(&[u8], &[u8]), libp2p::multiaddr::Error> {
        if input.len() < n {
            return Err(libp2p::multiaddr::Error::DataLessThanLen);
        }
        Ok(input.split_at(n))
    }
    let mut beginning: Vec<u8> = vec![];
    let mut end_bytes = addr.as_ref();

    let domain = loop {
        if end_bytes.is_empty() {
            break None;
        }
        let (protocol_id, rem_bytes) = unsigned_varint::decode::u32(end_bytes)?;
        if protocol_id == MULTIADDR_NUM {
            let (str_len, rem_bytes) = unsigned_varint::decode::usize(rem_bytes)?;
            let (data, rest) = split_at(str_len, rem_bytes)?;
            end_bytes = rest;
            break Some(str::from_utf8(data)?);
        }

        let (_, rem_bytes) = Protocol::from_bytes(end_bytes)?;
        beginning.extend_from_slice(&end_bytes[0..end_bytes.len() - rem_bytes.len()]);
        end_bytes = rem_bytes;
    };

    Ok((
        domain.map(ToOwned::to_owned),
        AmendableMultiAddr {
            beginning,
            end: end_bytes.to_owned(),
        },
    ))
}

impl libp2p::Transport for AlternetTransport {
    type Output = <BoxedTransport as Transport>::Output;
    type Error = Error<<BoxedTransport as Transport>::Error>;
    type ListenerUpgrade = futures_util::future::MapErr<
        <BoxedTransport as Transport>::ListenerUpgrade,
        fn(<BoxedTransport as Transport>::Error) -> Self::Error,
    >;
    type Dial = futures_util::future::Either<
        futures_util::future::MapErr<
            <BoxedTransport as Transport>::Dial,
            fn(<BoxedTransport as Transport>::Error) -> Self::Error,
        >,
        futures_core::future::BoxFuture<'static, Result<Self::Output, Self::Error>>,
    >;

    fn listen_on(
        &mut self,
        id: libp2p::core::transport::ListenerId,
        addr: libp2p::Multiaddr,
    ) -> Result<(), libp2p::TransportError<Self::Error>> {
        self.inner
            .lock()
            .listen_on(id, addr)
            .map_err(|e| e.map(Error::Transport))
    }

    fn remove_listener(&mut self, id: libp2p::core::transport::ListenerId) -> bool {
        self.inner.lock().remove_listener(id)
    }

    fn dial(
        &mut self,
        addr: libp2p::Multiaddr,
        dial_opts: libp2p::core::transport::DialOpts,
    ) -> Result<Self::Dial, libp2p::TransportError<Self::Error>> {
        Ok(self.do_dial(addr, dial_opts))
    }

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<libp2p::core::transport::TransportEvent<Self::ListenerUpgrade, Self::Error>>
    {
        let mut inner = self.inner.lock();
        libp2p::core::Transport::poll(std::pin::Pin::new(inner.deref_mut()), cx).map(|event| {
            event
                .map_upgrade(|upgr| upgr.map_err::<_, fn(_) -> _>(Error::Transport))
                .map_err(Error::Transport)
        })
    }
}
