use std::{fs::File, io::Write};
use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "config.toml";

pub fn config() -> ConfigFile {
    match std::fs::read_to_string(CONFIG_PATH) {
        Ok(string) => {
            toml::from_str::<ConfigFile>(&string).expect("Couldn't parse config file")
        },
        Err(error) => {
            if error.kind() == std::io::ErrorKind::PermissionDenied {
                panic!("Couldn't read config file: permission denied");
            }

            let config = ConfigFile::default();

            let mut file = File::create(CONFIG_PATH)
                .expect("Couldn't create config file");
            let _ = file.write_all(toml::to_string_pretty(&config)
                .expect("Couldn't serialise config file")
                .as_bytes());

            config
        },
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ConfigFile {
    pub server: ServerConfig,
    pub encryption: EncryptionConfig,
    pub logging: LoggingConfig,
    pub safety: SafetyConfig,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub protocol_id: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 25807,
            protocol_id: 123456789,
        }
    }
}


#[derive(Deserialize, Serialize, Debug)]
pub struct EncryptionConfig {
    pub certificate: String,
    pub privatekey: String,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            certificate: "certificate.pem".to_string(),
            privatekey: "privatekey.pem".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoggingConfig {
    pub verbose: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            verbose: false,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SafetyConfig {
    pub disconnect_time: f32,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            disconnect_time: 30.0,
        }
    }
}