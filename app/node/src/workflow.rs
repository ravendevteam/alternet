use super::*;

pub mod reservation;

pub trait Termination {
	fn is_ready_to_unmount(&self) -> bool;
}

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
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self {
		let Self(mut pool) = self;
		pool.extend(T::from_context(swarm, event, queue));
		
		Self(pool)
	}
}

impl<T> Workflow for Pool<T> 
where
	T: Workflow,
	T: Termination {
	fn next(
		self,
		swarm: &mut Swarm,
		event: &mut Event,
		queue: &mut dyn FnMut(Event)
	) -> Self {
		let Self(pool) = self;
		let mut out: Vec<_> = Vec::default();
		
		for workflow in pool {
			if workflow.is_ready_to_unmount() {
				continue
			}
			
			let workflow: T = workflow.next(swarm, event, queue);
			
			out.push(workflow);
		}
		
		Self(out)
	}
}