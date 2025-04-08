use serde::Deserialize;
use std::fs::File;
use std::{fs, io};

#[derive(Deserialize)]
pub struct Config {
    pub update_frequency: u16,
    pub backup_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backup_dir: 
        }
    }
}

impl Config {
    pub fn open() -> io::Result<()> {
        if fs::exists("/etc/bedrockd/config.toml").expect("Unable to check for config file!") {

        };
        Ok(())
    }
}
