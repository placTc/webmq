use async_trait::async_trait;

use crate::core::errors::WebMQError;

#[async_trait]
pub trait MessagingDispatcher<Q, D> {
    async fn publish(&mut self, queue: Q, data: D) -> Option<WebMQError>;
    async fn consume(&mut self, queue: Q) -> Result<D, WebMQError>;
}