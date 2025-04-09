use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::{fs, io};
use tokio::io::AsyncWriteExt;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub update_frequency: u16,
    pub backup_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backup_dir: "/opt/bedrockd".into(),
            update_frequency: 60,
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
}
