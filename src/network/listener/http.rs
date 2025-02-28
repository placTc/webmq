use std::{
    net::{Ipv4Addr, SocketAddrV4},
    pin::Pin,
    sync::Arc,
};

use async_trait::async_trait;
use http_body_util::Full;
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use log::{debug, error, warn};
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

use crate::core::{
    errors::WebMQError,
    traits::{AsyncStart, Service},
};

type Req = Request<Incoming>;
type Pb<T> = Pin<Box<T>>;
type Res = Result<Response<Full<Bytes>>, Err>;
type Err = WebMQError;
type Fut<T> = Pb<dyn Future<Output = T> + Send>;
type Svc = dyn Service<Input = Req, Output = Fut<Res>> + Send + Sync;

#[derive(Clone)]
pub struct HttpListener {
    tcp_listener: Arc<TcpListener>,
    service: Arc<Svc>,
}

impl HttpListener {
    pub async fn new(ip: Ipv4Addr, port: u16, service: Arc<Svc>) -> Result<Self, WebMQError> {
        let addr = SocketAddrV4::new(ip, port);

        let tcp_listener = Arc::new(match TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                error!("Could not create TCP listener: {e}");
                return Err(WebMQError::Unrecoverable);
            }
        });

        Ok(HttpListener {
            tcp_listener,
            service,
        })
    }

    fn spawn_handler_task(&self, tcp_stream: TcpStream) -> JoinHandle<()> {
        let handler = self.clone();
        tokio::task::spawn(async move {
            handler.handle_tcp_connection(tcp_stream).await;
        })
    }

    async fn handle_tcp_connection(&self, tcp_stream: TcpStream) {
        let svc = service_fn(async |request| self.service.call(request).await);
        let io = TokioIo::new(tcp_stream);
        if let Err(err) = http1::Builder::new()
            .timer(TokioTimer::new())
            .serve_connection(io, svc)
            .await
        {
            warn!("Error in service connection: {}", err);
        }
    }
}

#[async_trait]
impl AsyncStart for HttpListener {
    async fn start(&self) {
        loop {
            match self.tcp_listener.accept().await {
                Ok((tcp_stream, _addr)) => {
                    self.spawn_handler_task(tcp_stream);
                }
                Err(e) => {
                    debug!("Error during TCP connection: {e}");
                }
            };
        }
    }
}
