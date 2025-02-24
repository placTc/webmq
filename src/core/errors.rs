use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub enum WebMQError {
    Config(String),
    File(String),
    TLS(String),
    Unrecoverable,
}

impl Display for WebMQError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.message();
        write!(f, "{message}")
    }
}

impl WebMQError {
    fn message(&self) -> &str {
        match self {
            WebMQError::Config(msg) => msg.as_str(),
            WebMQError::File(msg) => msg.as_str(),
            WebMQError::TLS(msg) => msg.as_str(),
            WebMQError::Unrecoverable => "The program encountered an unrecoverable error.",
        }
    }
}

impl Error for WebMQError {}
