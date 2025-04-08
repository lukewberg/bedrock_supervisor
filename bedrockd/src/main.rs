mod backup;
mod cli;
mod config;
mod server;

use crate::backup::BackupManager;
use crate::cli::Cli;
use crate::config::Config;
use clap::Parser;
use fork::{Fork, daemon};
use std::fs::File;
use std::io::Read;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    if cli.daemon {
        if let Ok(Fork::Child) = daemon(false, false) {
            Command::new("bedrockd")
                .output()
                .expect("failed to execute process");
        }
    }

    // Parse config  file in /etc/bedrockd.conf
    let mut config_str = String::new();
    File::open("/etc/bedrockd.toml")?.read_to_string(&mut config_str)?;
    let config: Config = toml::from_str(config_str.as_str()).unwrap();

    let backup_manager = BackupManager::new(config.update_frequency, config.backup_dir.into());
    
    
    Ok(())
}
