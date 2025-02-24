use core::config::main::Settings;
use core::errors::WebMQError;
use network::tls::acceptor::create_tls_acceptor;
use std::convert::Infallible;
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;
use std::str::FromStr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use log::{debug, error, info, warn};
use tls_listener::TlsListener;
use tls_listener::rustls::{rustls, TlsAcceptor};
use tokio::net::TcpListener;

pub mod core;
pub mod utils;
pub mod network;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();
    debug!("Initialized logger");
    let _ = rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls cryptography provider");
    debug!("Installed rustls cryptography provider.");

    let config = Settings::load();

    let addr = SocketAddrV4::new(
        Ipv4Addr::from_str(config.network.ip.as_str())?,
        config.network.port,
    );

    let tls_acceptor = match create_tls_acceptor(
        Path::new(config.network.tls.certificate.as_str()),
        Path::new(config.network.tls.private_key.as_str()),
    ) {
        Ok(l) => l,
        Err(e) => {
            error!("{e}");
            return Err(WebMQError::Unrecoverable.into());
        }
    };
    info!("Initialized TLS acceptor");

    let mut listener: TlsListener<TcpListener, TlsAcceptor> =
        TlsListener::new(tls_acceptor, TcpListener::bind(addr).await?);    

    loop {
        let (stream, _) = match listener.accept().await {
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
