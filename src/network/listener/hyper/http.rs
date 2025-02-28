use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use async_trait::async_trait;

use log::{debug, error};
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

use crate::core::{errors::WebMQError, traits::AsyncStart};

use super::common::{HyperSvc, hyper_http1_handler};

#[derive(Clone)]
pub struct HttpListener {
    tcp_listener: Arc<TcpListener>,
    service: Arc<HyperSvc>,
}

impl HttpListener {
    pub async fn new(ip: Ipv4Addr, port: u16, service: Arc<HyperSvc>) -> Result<Self, WebMQError> {
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
        let service = self.service.clone();
        tokio::task::spawn(async move {
            hyper_http1_handler(tcp_stream, service.as_ref()).await;
        })
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
