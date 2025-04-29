use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::{fs, io};
use tokio::io::AsyncWriteExt;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub backup_frequency: u16,
    pub backup_dir: String,
    pub gRPC: Grpc,
    pub server: Server
}

#[derive(Serialize, Deserialize)]
pub struct Grpc {
    pub enabled: bool,
    pub port: u16,
    pub reflection: bool,
}

impl Default for Grpc {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 10000,
            reflection: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Server {
    pub path: String,
    pub binary: String,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            binary: "bedrock_server".to_string(),
            path: "/opt/minecraft".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backup_dir: "/opt/bedrockd".into(),
            backup_frequency: 60,
            gRPC: Grpc::default(),
            server: Server::default()
        }
    }
}

impl Config {
    pub fn open() -> io::Result<Self> {
        let mut config_str = String::new();
        let mut config_handle = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("/etc/bedrockd.conf")?;
        config_handle.read_to_string(&mut config_str)?;
        // Check that the contents aren't empty. If so, write default config
        if config_str.is_empty() {
            let default_config = Config::default();
            let contents = toml::ser::to_string_pretty(&default_config)
                .expect("Failed to serialize default config");
            config_handle.write_all(contents.as_bytes())?;
            config_handle.flush()?;
            Ok(default_config)
        } else {
            let config: Self = toml::from_str(config_str.as_str()).unwrap();
            Ok(config)
        }
    }

    pub fn create() -> io::Result<()> {
        let mut config_handle = OpenOptions::new()
            .write(true)
            .create(true)
            .open("/etc/bedrockd.conf")?;
        config_handle.write_all(toml::to_string_pretty(&Config::default()).unwrap().as_bytes())?;
        config_handle.flush()?;
        Ok(())
    }
}
