use super::*;

pub trait AnyHandler
where
    Self: Send {
    fn type_id(&self) -> std::any::TypeId;
    fn handle(&mut self, event: &dyn std::any::Any);
}

impl<T> AnyHandler for T
where
    T: Handler {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<T>()
    }

    fn handle(&mut self, event: &dyn std::any::Any) {
        let event: &<T as Handler>::Event = event.downcast_ref::<T::Event>().expect("event type should match");
        self.handle(event);
    }
}






