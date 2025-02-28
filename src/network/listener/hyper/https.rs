use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
    sync::Arc,
};

use async_trait::async_trait;
use log::{debug, error, info, warn};
use tls_listener::rustls::TlsAcceptor;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

use crate::{
    core::{config::tls::TlsSettings, errors::WebMQError, traits::AsyncStart},
    network::tls::acceptor::create_tls_acceptor,
};

use super::common::hyper_http1_handler;

use super::common::HyperSvc;

const TLS_CLIENT_HELLO_HEAD_SIZE: usize = 3;
const TLS_CLIENT_HELLO_HEAD: [u8; TLS_CLIENT_HELLO_HEAD_SIZE] = [0x16, 0x03, 0x01];

#[derive(Clone)]
pub struct HttpsListener {
    tls_acceptor: TlsAcceptor,
    tcp_listener: Arc<TcpListener>,
    service: Arc<HyperSvc>,
}

impl HttpsListener {
    pub async fn new(
        ip: Ipv4Addr,
        port: u16,
        tls_config: TlsSettings,
        service: Arc<HyperSvc>,
    ) -> Result<Self, WebMQError> {
        let addr = SocketAddrV4::new(ip, port);

        let tls_acceptor = match create_tls_acceptor(
            Path::new(tls_config.certificate.as_str()),
            Path::new(tls_config.private_key.as_str()),
        ) {
            Ok(l) => l,
            Err(e) => {
                error!("{e}");
                return Err(WebMQError::Unrecoverable);
            }
        };
        info!("Initialized TLS acceptor");

        let tcp_listener = Arc::new(match TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                error!("Could not create TCP listener: {e}");
                return Err(WebMQError::Unrecoverable);
            }
        });

        Ok(HttpsListener {
            tls_acceptor,
            tcp_listener,
            service,
        })
    }

    fn spawn_handler_task(&self, mut tcp_stream: TcpStream) -> JoinHandle<()> {
        let handler = self.clone();
        tokio::task::spawn(async move {
            if let Some(err) = is_tls(&tcp_stream).await {
                warn!("{err}");
                discard_stream(&mut tcp_stream).await;
                return;
            };

            handler.handle_tls_connection(tcp_stream).await;
        })
    }

    async fn handle_tls_connection(&self, tcp_stream: TcpStream) {
        match self.tls_acceptor.accept(tcp_stream).await {
            Ok(stream) => {
                hyper_http1_handler(stream, self.service.as_ref()).await;
            }
            Err(e) => warn!("Error during TLS handshake: {e}"),
        }

        debug!("closed stream");
    }
}

#[async_trait]
impl AsyncStart for HttpsListener {
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

async fn is_tls(stream: &TcpStream) -> Option<impl Error + use<>> {
    let mut buf = [0u8; TLS_CLIENT_HELLO_HEAD_SIZE];
    if stream.peek(&mut buf).await.is_ok() && buf != TLS_CLIENT_HELLO_HEAD {
        let Ok(peer_addr) = stream.peer_addr() else {
            return Some(WebMQError::TLS(
                "Received non-TLS data from peer".to_owned(),
            ));
        };

        return Some(WebMQError::TLS(format!(
            "Received non-TLS data from peer: {}",
            peer_addr
        )));
    }

    None
}

async fn discard_stream(stream: &mut TcpStream) {
    let peer_addr = match stream.peer_addr() {
        Ok(a) => a.to_string(),
        Err(_) => "{unknown}".to_string(),
    };

    match stream.shutdown().await {
        Ok(_) => {
            debug!("Forcibly shut down stream for peer: {peer_addr}");
        }
        Err(e) => {
            error!("Could not forcibly shut down stream for peer {peer_addr} due to error: {e}");
        }
    }
}
