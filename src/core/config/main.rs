use super::network::NetworkSettings;
use config::Config;
use log::{info, warn};

const CONFIGURATION_FILE: &str = "./configuration";

#[derive(Debug, serde::Deserialize, Default)]
pub struct Settings {
    #[serde(default = "NetworkSettings::default")]
    pub network: NetworkSettings,
}

impl Settings {
    pub fn load() -> Self {
        let raw_config = Config::builder()
            .add_source(config::File::with_name(CONFIGURATION_FILE))
            .build();

        match raw_config {
            Ok(config) => {
                let c = Self::try_deserialize_config(config);
                info!("Loaded configuration from {CONFIGURATION_FILE}");
                c
            }
            Err(error) => {
                warn!(
                    "Failed to load system configuration: {error}. Attempting to fall back to defaults."
                );
                Self::default()
            }
        }
    }

    fn try_deserialize_config(config: Config) -> Self {
        match config.try_deserialize() {
            Ok(config) => config,
            Err(error) => {
                warn!(
                    "Failed to parse system configuration: {error}. Attempting to fall back to defaults."
                );
                Self::default()
            }
        }
    }
}
