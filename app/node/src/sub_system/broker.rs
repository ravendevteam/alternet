use super::*;

pub struct Verify;

pub enum State {
	Unset,
	Ready
}

pub struct Broker<A, B> {
	erc_20: A,
	dns: B,
	state: State
}

impl<A, B> SubSystem for Broker<A, B> 
where
	B: Dns {
	fn receive(
		&mut self, 
		swarm: &mut Swarm, 
		event: &mut Event, 
		queue: &mut dyn FnMut(Event)
	) {
		if let Some(router::InboundBytes {
			src,
			content
		}) = event.downcast_ref() {
			
		}
		
		tokio::runtime::Handle::current().block_on(async {
			match event.downcast_ref() {
				// check for verify ivent
				Some(Verify) => {
					
				},
				None => return
			}
			
			match self.state {
				State::Unset => {
					
				},
				State::Ready => return
			}
		});
		

		
    	#[cfg(any(feature = "client"))] {

       		// blocks, ideally will need to architect a way to not do this
			tokio::runtime::Handle::current().block_on(async {
				// check how much balance the client has, check the pk
				let balance: Balance = self.dns.locked_balance_of().await.unwrap();
				
				// check threshold and time until last pool will unlock
				
				self.dns.lock(Balance::from(20000), Duration::from(60000)).await.ok();
			});

			
       
       		
       		
     		// lock up pool
     	}
      
      	#[cfg(any(feature = "relay"))] {
       		// pools that are about to expire in less than x time, will be ignored, this time may be set by the relay
       
       
       		tokio::spawn(async move {
         		self.dns.pool().await;
           	
           		// after verifying approve the service
           		emit()
         	});
         
       		// detect when you have forwarded content
         	//  
         	// remember it (if the server doesnt return proof of your work, blacklist or reduce reputation)
       	}
        
        #[cfg(any(feature = "server"))] {
        	// received, from who, and if relay is there
         	// sign proof, and return back to the relay
          	// 
        }
	}
}