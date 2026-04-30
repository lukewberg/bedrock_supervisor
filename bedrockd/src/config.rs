use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub grpc: Grpc,
    pub server: Server,
    pub backup: Backup,
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
            reflection: true,
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

#[derive(Serialize, Deserialize, Clone)]
pub enum BackupFrequency {
    Minute,
    Hourly,
    Daily,
    Weekly,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupSchedule {
    pub frequency: BackupFrequency,
    pub value: u16,
    pub limit: u16,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Backup {
    pub path: String,
    pub level_name: String,
    pub schedule: Vec<BackupSchedule>,
}

impl Default for Backup {
    fn default() -> Self {
        Self {
            schedule: vec![BackupSchedule {
                frequency: BackupFrequency::Minute,
                value: 5,
                limit: 15,
                enabled: true,
            }],
            path: "/opt/minecraft/backup".to_string(),
            level_name: "Bedrock level".to_string(),
        }
    }
}

impl Config {
    pub fn open() -> io::Result<Self> {
        let mut config_str = String::new();
        let mut config_handle = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
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
            .truncate(true)
            .create(true)
            .open("/etc/bedrockd.conf")?;
        config_handle.write_all(
            toml::to_string_pretty(&Config::default())
                .unwrap()
                .as_bytes(),
        )?;
        config_handle.flush()?;
        Ok(())
    }
}
