use super::*;

pub mod reservation;

pub trait Unique {
	fn key(&self) -> &str;
}

pub trait Saga 
where
	Self: Unique {
	fn next(
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self;
}

pub struct Pool<T>(Vec<T>);

impl<T> Pool<T> 
where
	T: Saga {
	pub fn fund(
		&mut self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) {
		for saga in self.0 {
			saga.next(swarm, event, queue);
		}
	}
}