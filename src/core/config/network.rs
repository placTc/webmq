use crate::core::config::tls::TLSSettings;

const DEFAULT_IP: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8060;

#[derive(Debug, serde::Deserialize)]
pub struct NetworkSettings {
    #[serde(default = "NetworkSettings::default_ip")]
    pub ip: String,
    #[serde(default = "NetworkSettings::default_port")]
    pub port: u16,
    #[serde(default = "TLSSettings::default")]
    pub tls: TLSSettings,
}

impl NetworkSettings {
    fn default_ip() -> String {
        DEFAULT_IP.to_string()
    }

    fn default_port() -> u16 {
        DEFAULT_PORT
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            ip: Self::default_ip(),
            port: Self::default_port(),
            tls: TLSSettings::default(),
        }
    }
}
