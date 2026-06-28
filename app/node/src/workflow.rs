use super::*;

pub mod reservation;

pub trait FromContext 
where
	Self: Sized {
	fn from_context(
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Vec<Self>;
}

pub trait Workflow {
	fn next(
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self;
	
	fn is_ready_to_unmount(&self) -> bool {
		false
	}
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(derive_more::From)]
pub struct Pool<T>(Vec<T>);

impl<T> Pool<T> 
where
	T: FromContext {
	pub fn generate(
		&mut self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) {
		self.0.extend(T::from_context(swarm, event, queue));
	}
}

impl<T> Pool<T>
where
	T: Workflow {
	pub fn next(
		&mut self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) {		
		let old_pool: Vec<_> = std::mem::take(&mut self.0);
		let old_pool_len: usize = old_pool.len();
		let mut new_pool: Vec<_> = Vec::with_capacity(old_pool_len);
		for workflow in old_pool {
			if workflow.is_ready_to_unmount() {
				continue
			}
			let workflow: T = workflow.next(swarm, event, queue);
			new_pool.push(workflow);
		}
		self.0 = new_pool;
	}
}