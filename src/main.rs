use core::errors::WebMQError;
use core::traits::AsyncStart;
use core::{config::main::Settings, traits::Service};
use std::pin::Pin;
use std::sync::Arc;
use std::{error::Error, net::Ipv4Addr, str::FromStr};

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use log::{debug, error};
use network::listener::https::HttpsListener;
use tls_listener::rustls::rustls;

pub mod core;
pub mod network;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();
    debug!("Initialized logger");
    let _ = rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls cryptography provider");
    debug!("Installed rustls cryptography provider.");

    let config = Settings::load();

    let Ok(ip) = Ipv4Addr::from_str(config.network.ip.as_str()) else {
        error!("Couldn't parse IP adress");
        return Err(WebMQError::Unrecoverable.into());
    };

    let listener: Box<dyn AsyncStart> = match HttpsListener::new(
        ip,
        config.network.port,
        config.network.tls,
        Arc::new(HyperService {}),
    )
    .await
    {
        Ok(l) => Box::new(l),
        Err(e) => {
            error!("Couldn't create listener: {e}");
            return Err(WebMQError::Unrecoverable.into());
        }
    };

    listener.start().await;

    Ok(())
}

struct HyperService {}

type Res = Response<Full<Bytes>>;

impl Service for HyperService {
    type Input = Request<Incoming>;
    type Output = Pin<Box<dyn Future<Output = Result<Res, WebMQError>> + Send>>;

    fn call(&self, _: Self::Input) -> Self::Output {
        Box::pin(async move {
            Ok(Response::builder().header("Connection", "close")
                .body(Full::new(Bytes::from("Hello, World!")))
                .unwrap())
        })
    }
}
