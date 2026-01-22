use crate::prelude::*;
use futures::channel::mpsc::{Receiver, Sender};

pub enum Request {
    Resolve(resolve::Request),
    Register(register::Request),
    Deregister(deregister::Request),
}
pub type RequestSender = Sender<Request>;
pub type RequestReceiver = Receiver<Request>;

#[derive(Clone)]
pub struct Control {
    control_sender: RequestSender,
}

#[ouroboros::self_referencing]
struct CommandFut {
    sender: RequestSender,
    // todo: check if this is actually covariant - right now i think so
    // https://docs.rs/ouroboros/latest/ouroboros/attr.self_referencing.html#covariance
    #[covariant]
    #[borrows(mut sender)]
    fut: futures::sink::Send<'this, RequestSender, Request>,
}
impl Future for CommandFut {
    type Output = Result<(), <RequestSender as Sink<Request>>::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.get_mut().with_fut_mut(|fut| fut.poll_unpin(cx))
    }
}
fn command_fut(sender: &RequestSender, request: Request) -> CommandFut {
    CommandFut::new(sender.clone(), move |sender| sender.send(request))
}
impl Control {
    #[allow(private_interfaces)]
    #[must_use = "futures do nothing unless polled"]
    pub fn command(&self, request: Request) -> CommandFut {
        command_fut(&self.control_sender, request)
    }
}

// helper for dealing with chaining stuff after Control::command
#[must_use = "futures do nothing unless polled"]
fn inspect_and_then<Fut>(cmd: CommandFut, then: Fut) -> InspectAndThen<Fut> {
    InspectAndThen {
        cmd: Some(cmd),
        then,
    }
}

#[must_use = "futures do nothing unless polled"]
pub(crate) struct InspectAndThen<Fut> {
    cmd: Option<CommandFut>,
    then: Fut,
}
impl<T: Unpin> InspectAndThen<T> {
    fn poll_cmd_done(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> bool {
        if let Some(cmd) = &mut self.cmd {
            if let std::task::Poll::Ready(res) = cmd.poll_unpin(cx) {
                if let Err(err) = res {
                    // todo tracing info that trying to resolve with behaviour receiver dropped
                    // todo: figure out whether e can ever be SendError::Full
                    _ = err;
                }
            } else {
                return false;
            }
        }
        true
    }
}
impl<Fut> Future for InspectAndThen<Fut>
where
    Fut: Future + Unpin,
{
    type Output = Fut::Output;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !Self::poll_cmd_done(self.as_mut(), cx) {
            return std::task::Poll::Pending;
        }
        // only poll then after cmd has finished
        self.then.poll_unpin(cx)
    }
}
impl<St> stream::Stream for InspectAndThen<St>
where
    St: stream::Stream + Unpin,
{
    type Item = St::Item;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if !Self::poll_cmd_done(self.as_mut(), cx) {
            return std::task::Poll::Pending;
        }
        self.then.poll_next_unpin(cx)
    }
}

pub mod resolve {
    use crate::{control::inspect_and_then, prelude::*};
    use futures::channel::oneshot::{Receiver, Sender, channel};

    // todo: maybe this shouldn't be a result and instead just a stream of Multiaddr?
    // i doubt code will care about an error - they can just detect when there is no more
    // addresses and stop, as well as when there are no addresses at all they know it isn't registered
    pub type Response = ::std::result::Result<Vec<Multiaddr>, hickory_resolver::proto::ProtoError>;
    pub type ResponseSender = Sender<Response>;
    pub type ResponseReceiver = Receiver<Response>;

    pub struct Request {
        pub domain: hickory_resolver::Name,
        pub responder: ResponseSender,
    }

    pub(crate) type ResolveFut = super::InspectAndThen<ResponseReceiver>;

    impl super::Control {
        #[allow(private_interfaces)]
        #[must_use = "futures do nothing unless polled"]
        pub fn resolve(&self, domain: hickory_resolver::Name) -> ResolveFut {
            let (sender, receiver) = channel();
            let command_fut = self.command(super::Request::Resolve(Request {
                domain,
                responder: sender,
            }));
            inspect_and_then(command_fut, receiver)
        }
    }
}

pub mod register {
    use crate::prelude::*;
    use futures::channel::oneshot::{Receiver, Sender, channel};

    // todo:
    // - add reason for registration failure
    #[derive(Debug)]
    pub struct Error;
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("error registering Alternet Domain")
        }
    }
    impl std::error::Error for Error {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            None
        }
    }

    pub type Response = ::std::result::Result<(), Error>;
    pub type ResponseReceiver = Receiver<Response>;
    pub type ResponseSender = Sender<Response>;

    pub struct Request {
        pub domain: String,
        pub response: ResponseSender,
    }

    pub(crate) type RegisterFut = super::InspectAndThen<ResponseReceiver>;

    impl super::Control {
        #[allow(private_interfaces)]
        #[must_use = "futures do nothing unless polled"]
        pub fn register(&self, domain: String) -> RegisterFut {
            let (sender, receiver) = channel();
            let command_fut = self.command(super::Request::Register(Request {
                domain,
                response: sender,
            }));
            super::inspect_and_then(command_fut, receiver)
        }
    }
}

pub mod deregister {
    use libp2p::futures;

    use crate::control::inspect_and_then;

    // todo:
    // - add reason for registration failure
    #[derive(Debug)]
    pub struct Error;
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("error registering Alternet Domain")
        }
    }
    impl std::error::Error for Error {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            None
        }
    }

    pub(crate) type DeregisterFut = super::InspectAndThen<futures::future::Ready<()>>;

    pub struct Request {
        pub domain: String,
    }

    impl super::Control {
        #[allow(private_interfaces)]
        #[must_use = "futures do nothing unless polled"]
        pub fn deregister(&self, domain: String) -> DeregisterFut {
            let command_fut = self.command(super::Request::Deregister(Request { domain }));
            inspect_and_then(command_fut, futures::future::ready(()))
        }
    }
}
