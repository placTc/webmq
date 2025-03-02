use std::error::Error;

use async_trait::async_trait;
#[async_trait]
pub trait AsyncStart {
    async fn start(&self);
}

#[async_trait]
pub trait Adapter {
    type Input;
    type Output;

    async fn call(&self, input: Self::Input) -> Self::Output;
}


#[async_trait]
pub trait AsyncQueue<T> {
    async fn pop(&mut self) -> Result<T, Box<dyn Error>>;
    async fn push(&mut self, data: T) -> Option<Box<dyn Error>>;
}