use tokio::sync::oneshot;

pub type Channel<T> = (Promise<T>, Resolver<T>);

pub struct Promise<T> {
    rx: oneshot::Receiver<T>
}

impl<A> Promise<A> {
    pub fn new() -> Channel<A> {
        let (sx, rx) = oneshot::channel();
        let sx: Resolver<A> = Resolver {
            sx
        };
        let rx: Promise<A> = Promise {
            rx
        };
        (rx, sx)
    }

    pub fn from_closure<B>(f: B) -> Self 
    where
        A: Send + 'static,
        B: FnOnce(Resolve<A>) {
        let (rx, sx) = Self::new();
        let resolve = move |v: A| {
            sx.resolve(v);
        };
        let resolve: Box<_> = Box::new(resolve);
        f(resolve);
        rx
    }

    pub fn spawn<B, C>(f: B) -> Self 
    where
        A: Send,
        A: 'static,
        B: FnOnce(Resolve<A>) -> C,
        B: Send,
        B: 'static,
        C: Future<Output = ()>,
        C: Send,
        C: 'static {
        let (rx, sx) = Self::new();
        let resolve = move |v: A| {
            sx.resolve(v);
        };
        let resolve: Box<_> = Box::new(resolve);
        tokio::spawn(async move {
            f(resolve).await;
        });
        rx
    }
}

impl<T> IntoFuture for Promise<T> {
    type Output = Result<T, oneshot::error::RecvError>;
    type IntoFuture = oneshot::Receiver<T>;

    fn into_future(self) -> Self::IntoFuture {
        self.rx
    }
}



pub type Resolve<T> = Box<dyn FnOnce(T) + Send>;

pub struct Resolver<T> {
    sx: oneshot::Sender<T>
}

impl<T> Resolver<T> {
    pub fn resolve(self, v: T) {
        let _ = self.sx.send(v);
    }
}



fn t() -> Promise<u8> {
    let n = Promise::<u8>::spawn(|resolve| async move {

        resolve(200);
    });

    // still a promise but this operations resolve after n is available
    // if not these are no ops
    let n = n + 2;
    
    n
}