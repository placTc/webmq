use async_trait::async_trait;
#[async_trait]
pub trait AsyncStart {
    async fn start(&self);
}

pub trait Service {
    type Input;
    type Output;

    fn call(&self, input: Self::Input) -> Self::Output;
}
