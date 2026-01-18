use super::*;

#[async_trait]
pub trait Runtime 
where
    Self: Send {
    async fn launch(self: Box<Self>) -> Result<()>;
}

pub struct Network {
    runtimes: Vec<Box<dyn Runtime>>
}

impl Network {
    pub fn new(runtimes: impl Into<Vec<Box<dyn Runtime>>>) -> Self {
        let runtimes: Vec<Box<dyn  Runtime>> = runtimes.into();
        Self {
            runtimes
        }
    }
}

impl Network {
    pub fn add_runtime(mut self, runtime: impl Runtime + 'static) -> Self {
        let runtime: Box<dyn Runtime> = Box::new(runtime);
        self.runtimes.push(runtime);
        self
    }

    pub async fn connect(mut self) -> Result<()> {
        for runtime in self.runtimes.drain(..) {
            let join_handle: tokio::task::JoinHandle<_> = tokio::spawn(async move {
                if let Err(error) = runtime.launch().await {
                    eprintln!("Runtime failed: {:?}", error);
                }
            });
        }
        future::future::pending::<()>().await;
        Ok(())
    }
}

impl Default for Network {
    fn default() -> Self {
        let runtimes: Vec<Box<dyn Runtime>> = vec![];
        Self {
            runtimes
        }
    }
}