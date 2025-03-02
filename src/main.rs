use core::errors::WebMQError;
use core::traits::{AsyncQueue, AsyncStart};
use core::config::main::Settings;
use std::sync::Arc;
use std::{error::Error, net::Ipv4Addr, str::FromStr};

use adapter::hyper_adapter::HyperAdapter;
use data::memory_queue::MemoryQueue;
use log::{debug, error};
use messaging::base_dispatcher::BaseMessagingDispatcher;
use network::listener::hyper::https::HttpsListener;
use tls_listener::rustls::rustls;
use tokio::sync::Mutex;

pub mod core;
pub mod network;
pub mod utils;
pub mod adapter;
pub mod messaging;
pub mod data;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();
    debug!("Initialized logger");
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls cryptography provider");
    debug!("Installed rustls cryptography provider.");

    let config = Settings::load();

    let Ok(ip) = Ipv4Addr::from_str(config.network.ip.as_str()) else {
        error!("Couldn't parse IP adress");
        return Err(WebMQError::Unrecoverable.into());
    };

    let dispatcher = Box::new(BaseMessagingDispatcher::new(Box::pin(create_memory_queue)));
    let adapter = HyperAdapter {
        dispatcher: Mutex::new(dispatcher)
    };
    let listener: Box<dyn AsyncStart> = match HttpsListener::new(
        ip,
        config.network.port,
        config.network.tls,
        Arc::new(adapter),
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

fn create_memory_queue() -> Box<dyn AsyncQueue<Vec<u8>> + Send> {
    Box::new(MemoryQueue::new())
}