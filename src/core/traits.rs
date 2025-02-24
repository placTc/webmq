use async_trait::async_trait;

#[async_trait]
pub trait AsyncStart {
    async fn start(&mut self);
}
