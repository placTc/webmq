use async_trait::async_trait;
use http_body_util::{BodyExt, Full, };
use hyper::{body::{Bytes, Incoming}, Method, Request, Response};
use tokio::sync::Mutex;

use crate::{core::{errors::WebMQError, traits::Adapter}, messaging::dispatcher::MessagingDispatcher};

pub struct HyperAdapter {
    pub dispatcher: Mutex<Box<dyn MessagingDispatcher<String, Vec<u8>> + Send + Sync>>
}

type Res = Response<Full<Bytes>>;

#[async_trait]
impl Adapter for HyperAdapter {
    type Input = Request<Incoming>;
    type Output = Result<Res, WebMQError>;

    async fn call(&self, request: Self::Input) -> Self::Output {
        
        match request.method() {
            &Method::GET => {
                Ok(Response::builder()
                        .body(Full::new(Bytes::from("Hello, GET!")))
                        .unwrap())
            }
            &Method::POST if request.uri().path().starts_with("/queue") => {
                let Some(queue) = request.uri().path().strip_prefix("/").unwrap().split("/").nth(1) else {
                    return Ok(Response::builder().status(404).body(empty_body()).unwrap());
                };
                
                let q = queue.to_owned();
                let b = request.collect().await.unwrap().to_bytes();
                self.dispatcher.lock().await.publish(q.clone(), b.to_vec()).await;
                return Ok(Response::builder()
                    .body(Full::new(Bytes::from(format!("Posted message to queue {}", q))))
                    .unwrap())
            },
            _ => {
                Ok(Response::builder().status(404).body(empty_body()).unwrap())
            }
        }
    }
}

fn empty_body() -> Full<Bytes> {
    Full::from("")
}