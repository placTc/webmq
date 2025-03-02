use async_trait::async_trait;
use http_body_util::{BodyExt, Full, };
use hyper::{body::{Bytes, Incoming}, Method, Request, Response};
use log::{info, warn};
use tokio::sync::Mutex;

use crate::{core::{errors::WebMQError, traits::Adapter}, core::traits::MessagingDispatcher};

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
            &Method::GET if request.uri().path().starts_with("/queue") => {
                let Some(queue) = request.uri().path().strip_prefix("/").unwrap().split("/").nth(1) else {
                    return Ok(response_404());
                };

                let q = queue.to_owned();
                let res = self.dispatcher.lock().await.consume(q.clone()).await;
                match res {
                    Ok(res) => {
                        info!("Consumed message on queue {q}");
                        Ok(Response::builder().body(Full::new(Bytes::from(res))).unwrap())
                    },
                    Err(res) => {
                        warn!("{res}");
                        Ok(Response::builder().status(204).body(empty_body()).unwrap())
                    } 
                        
                }

            }
            &Method::POST if request.uri().path().starts_with("/queue") => {
                let Some(queue) = request.uri().path().strip_prefix("/").unwrap().split("/").nth(1) else {
                    return Ok(response_404());
                };
                
                let q = queue.to_owned();
                let b = request.collect().await.unwrap().to_bytes();
                self.dispatcher.lock().await.publish(q.clone(), b.to_vec()).await;
                info!("Posted message on queue {q}");
                return Ok(Response::builder()
                    .status(202)
                    .body(empty_body())
                    .unwrap())
            },
            _ => {
                Ok(response_404())
            }
        }
    }
}

fn empty_body() -> Full<Bytes> {
    Full::from("")
}

fn response_404() -> Res {
    Response::builder().status(404).body(empty_body()).unwrap()
}