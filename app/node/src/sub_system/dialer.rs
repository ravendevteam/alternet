use super::*;

pub struct Dialer;

impl SubSystem for Dialer {
    fn receive(&mut self, swarm: &mut Swarm, event: &mut Event, queue: &mut dyn FnMut(Event)) {
        let Some(grpc::Dial{
            addr,
            completed
        }) = event.downcast_mut() else {
            return
        };
        let addr: libp2p::Multiaddr = addr.to_owned();
        let dial_op = swarm.dial(
            addr.to_owned()
        );
        let completed = completed.to_owned();
        tokio::spawn(async move {
            let mut completed: tokio::sync::MutexGuard<_> = completed.lock().await;
            match dial_op {
                Ok(_) => {
                    log::info!("successfully dialed {:?}", addr);
                    *completed = Some(true);
                },
                Err(_) => {
                    log::error!("failed to dial {:?}", addr);
                    *completed = Some(false);
                }
            }
        });
    }
}