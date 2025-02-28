use std::pin::Pin;

use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, Request, Response};

use crate::core::{errors::WebMQError, traits::Adapter};

pub struct HyperAdapter {}

type Res = Response<Full<Bytes>>;

impl Adapter for HyperAdapter {
    type Input = Request<Incoming>;
    type Output = Pin<Box<dyn Future<Output = Result<Res, WebMQError>> + Send>>;

    fn call(&self, _: Self::Input) -> Self::Output {
        Box::pin(async move {
            Ok(Response::builder()
                .header("Connection", "Keep-Alive")
                .body(Full::new(Bytes::from("Hello, World!")))
                .unwrap())
        })
    }
}