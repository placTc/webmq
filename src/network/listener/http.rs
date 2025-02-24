use std::{convert::Infallible, error::Error, net::{Ipv4Addr, SocketAddrV4}, path::Path};

use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use log::{error, info, warn};
use tls_listener::{rustls::TlsAcceptor, TlsListener};
use tokio::net::TcpListener;

use crate::{core::{config::tls::TLSSettings, errors::WebMQError, traits::AsyncStart}, network::tls::acceptor::create_tls_acceptor};

pub struct HTTPListener {
    listener: TlsListener<TcpListener, TlsAcceptor>
}

impl HTTPListener {
    pub async fn new(ip: Ipv4Addr, port: u16, tls_config: TLSSettings) -> Result<Self, Box<dyn Error>> {
        let addr = SocketAddrV4::new(ip, port);
    
        let tls_acceptor = match create_tls_acceptor(
            Path::new(tls_config.certificate.as_str()),
            Path::new(tls_config.private_key.as_str()),
        ) {
            Ok(l) => l,
            Err(e) => {
                error!("{e}");
                return Err(WebMQError::Unrecoverable.into());
            }
        };
        info!("Initialized TLS acceptor");
    
        let listener: TlsListener<TcpListener, TlsAcceptor> = TlsListener::new(tls_acceptor, TcpListener::bind(addr).await?);    
    
        Ok(HTTPListener {
            listener
        })
    }
}

impl AsyncStart for HTTPListener {
    async fn start(&mut self) {
        loop {
            let (stream, _) = match self.listener.accept().await {
                Ok(r) => r,
                Err(e) => {
                    let Some(peer_addr) = e.peer_addr() else {
                        warn!("{e}");
                        continue;
                    };
                    warn!("Error during TLS handshake for peer {peer_addr}: {e}");
                    continue;
                }
            };
    
            let io = TokioIo::new(stream);
    
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .timer(TokioTimer::new())
                    .serve_connection(io, service_fn(hello))
                    .await
                {
                    warn!("Error service connection: {:?}", err);
                }
            });
        }
    }
}


async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new("Hello, World!".as_bytes().into())))
}
