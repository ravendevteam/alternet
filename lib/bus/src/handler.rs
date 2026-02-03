pub trait Handler
where
    Self: Send,
    Self: 'static {
    type Event;

    fn handle(&mut self, event: &Self::Event);
}