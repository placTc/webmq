use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
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
use log::{debug, error, info, warn};
use tls_listener::rustls::TlsAcceptor;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

use crate::{
    core::{
        config::tls::TlsSettings,
        errors::WebMQError,
        traits::{AsyncStart, Service},
    },
    network::tls::acceptor::create_tls_acceptor,
};

const TLS_CLIENT_HELLO_HEAD_SIZE: usize = 3;
const TLS_CLIENT_HELLO_HEAD: [u8; TLS_CLIENT_HELLO_HEAD_SIZE] = [0x16, 0x03, 0x01];

type Req = Request<Incoming>;
type Pb<T> = Pin<Box<T>>;
type Res = Result<Response<Full<Bytes>>, Err>;
type Err = WebMQError;
type Fut<T> = Pb<dyn Future<Output = T> + Send>;
type Svc = Arc<dyn Service<Input = Req, Output = Fut<Res>> + Send + Sync>;

pub struct HttpsListener {
    tls_acceptor: TlsAcceptor,
    tcp_listener: Arc<TcpListener>,
    service: Svc,
}

impl HttpsListener {
    pub async fn new(
        ip: Ipv4Addr,
        port: u16,
        tls_config: TlsSettings,
        service: Svc,
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
}

fn spawn_handler_task(
    mut tcp_stream: TcpStream,
    tls_acceptor: TlsAcceptor,
    service: Svc,
) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        if let Some(err) = confirm_request_as_tls(&tcp_stream).await {
            warn!("{err}");
            discard_stream(&mut tcp_stream).await;
            return;
        };

        handle_tls_connection(tcp_stream, tls_acceptor, service).await;
    })
}

async fn handle_tls_connection(tcp_stream: TcpStream, tls_acceptor: TlsAcceptor, service: Svc) {
    let svc = service_fn(async |request| service.call(request).await);
    match tls_acceptor.accept(tcp_stream).await {
        Ok(stream) => {
            let io = TokioIo::new(stream);
            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, svc)
                .await
            {
                warn!("Error in service connection: {}", err);
            }
        }
        Err(e) => warn!("Error during TLS handshake: {e}"),
    }

    debug!("closed stream");
}

#[async_trait]
impl AsyncStart for HttpsListener {
    async fn start(&self) {
        loop {
            match self.tcp_listener.accept().await {
                Ok((tcp_stream, _addr)) => {
                    spawn_handler_task(tcp_stream, self.tls_acceptor.clone(), self.service.clone());
                }
                Err(e) => {
                    debug!("Error during TCP connection: {e}");
                }
            };
        }
    }
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
