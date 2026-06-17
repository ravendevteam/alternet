use super::*;

pub mod proof_request;
pub mod reservation;

pub trait Unique {
	fn key(&self) -> &str;
}

pub trait Termination {
	fn clean_up(&self) -> bool;
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

pub struct Terminatable<T>(T);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct Pool<T>(Vec<T>);

impl<T> Pool<T> 
where
	T: Saga,
	T: Termination {
	pub fn next(
		&mut self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) {
		for saga in self.0 {
			if saga.clean_up() {
				// remove and skip
				
				continue
			}
			
			saga.next(swarm, event, queue);
		}
	}
}