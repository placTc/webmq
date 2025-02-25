use std::{
    convert::Infallible,
    error::Error,
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
    sync::Arc,
};

use async_trait::async_trait;
use http_body_util::Full;
use hyper::{Request, Response, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::{TokioIo, TokioTimer};
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

const TLS_CLIENT_HELLO_HEAD_SIZE: usize = 3;
const TLS_CLIENT_HELLO_HEAD: [u8; TLS_CLIENT_HELLO_HEAD_SIZE] = [0x16, 0x03, 0x01];

pub struct HttpsListener {
    tls_acceptor: TlsAcceptor,
    tcp_listener: Arc<TcpListener>,
}

impl HttpsListener {
    pub async fn new(ip: Ipv4Addr, port: u16, tls_config: TlsSettings) -> Result<Self, WebMQError> {
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
        })
    }

    fn spawn_handler_task(&self, mut tcp_stream: TcpStream) -> JoinHandle<()> {
        let self_clone = self.clone();
        tokio::task::spawn(async move {
            if let Some(err) = confirm_request_as_tls(&tcp_stream).await {
                warn!("{err}");
                discard_stream(&mut tcp_stream).await;
                return;
            };

            self_clone.handle_tls_connection(tcp_stream).await;
        })
    }

    async fn handle_tls_connection(&self, tcp_stream: TcpStream) {
        match self.tls_acceptor.accept(tcp_stream).await {
            Ok(stream) => {
                let io = TokioIo::new(stream);
                if let Err(err) = http1::Builder::new()
                    .timer(TokioTimer::new())
                    .serve_connection(io, service_fn(hello))
                    .await
                {
                    warn!("Error in service connection: {:?}", err);
                }
            }
            Err(e) => warn!("Error during TLS handshake: {e}"),
        }
    }
}

#[async_trait]
impl AsyncStart for HttpsListener {
    async fn start(&mut self) {
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

impl Clone for HttpsListener {
    fn clone(&self) -> Self {
        HttpsListener {
            tcp_listener: self.tcp_listener.clone(),
            tls_acceptor: self.tls_acceptor.clone(),
        }
    }
}

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .body(Full::new("Hello, World!".as_bytes().into()))
        .unwrap())
}

async fn confirm_request_as_tls(stream: &TcpStream) -> Option<impl Error + use<>> {
    let mut buf = [0u8; TLS_CLIENT_HELLO_HEAD_SIZE];
    if let Ok(_) = stream.peek(&mut buf).await {
        if buf != TLS_CLIENT_HELLO_HEAD {
            let Ok(peer_addr) = stream.peer_addr() else {
                return Some(WebMQError::TLS(
                    "Received non-TLS data from peer".to_owned(),
                ));
            };

            return Some(WebMQError::TLS(format!(
                "Received non-TLS data from peer: {}",
                peer_addr.to_string()
            )));
        }
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
