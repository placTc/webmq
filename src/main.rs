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

    Ok(())
}
