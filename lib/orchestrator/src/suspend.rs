use super::*;

pub struct Suspend<T> 
where
    T: Node {
    tokens: Vec<Runtime<T>>
}

impl<T> Suspend<T> 
where
    T: Node {
    pub async fn wait_for(&self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await;
    }

    pub async fn wait(&self) {
        future::future::pending::<()>().await;
    }

    pub async fn wait_for_ctrl_c(&self) {
        tokio::signal::ctrl_c().await.expect("failed to listen for ctrl_c");
    }

    pub fn shutdown_all(&mut self) {
        self.tokens.clear();
    }
}

impl<T> From<Vec<Runtime<T>>> for Suspend<T> 
where
    T: Node {
    fn from(value: Vec<Runtime<T>>) -> Self {
        let tokens: Vec<_> = value;
        Self {
            tokens
        }
    }
}