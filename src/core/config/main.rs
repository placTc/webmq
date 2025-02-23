use super::network::NetworkSettings;
use config::Config;
use log::warn;

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    #[serde(default = "NetworkSettings::default")]
    pub network: NetworkSettings
}

impl Default for Settings {
    fn default() -> Self {
        Self { network: NetworkSettings::default() }
    }
}

impl Settings {
    pub fn load() -> Self {
        let raw_config = Config::builder()
        .add_source(config::File::with_name("./configuration"))
        .build();
    
    
        match raw_config {
            Ok(config) => {
                Self::try_deserialize_config(config)
            },
            Err(error) => {
                warn!("Failed to load system configuration: {error}. Falling back to default.");
                Self::default()
            }
        }
    }

    fn try_deserialize_config(config: Config) -> Self {
        match config.try_deserialize() {
            Ok(config) => config,
            Err(error) => {
                warn!("Failed to parse system configuration: {error}. Falling back to default");
                Self::default()
            }
        }
    }
}


