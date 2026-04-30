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
            .open("/etc/bedrockd.conf")
            .map_err(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "Config file not found. Run `bedrockd --config` as root to create it.",
                    )
                } else {
                    e
                }
            })?;
        config_handle.read_to_string(&mut config_str)?;
        toml::from_str(&config_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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
