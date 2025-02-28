use std::pin::Pin;

use http_body_util::Full;
use hyper::{body::{Bytes, Incoming}, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use log::warn;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::core::{errors::WebMQError, traits::Service};

type Err = WebMQError;
type Pb<T> = Pin<Box<T>>;
type Fut<T> = Pb<dyn Future<Output = T> + Send>;

pub type Req = Request<Incoming>;
pub type Res = Result<Response<Full<Bytes>>, Err>;
pub type HyperSvc = dyn Service<Input = Req, Output = Fut<Res>> + Send + Sync;

pub async fn hyper_http1_handler<S>(stream: S, service: &HyperSvc)
where
    S: AsyncRead + AsyncWrite + Unpin
{
    let svc = service_fn(async |request| service.call(request).await);
    let io = TokioIo::new(stream);
    if let Err(err) = http1::Builder::new()
        .timer(TokioTimer::new())

        .serve_connection(io, svc)
        .await
    {
        warn!("Error in service connection: {}", err);
    }
}