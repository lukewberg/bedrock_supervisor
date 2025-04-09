mod backup;
mod cli;
mod config;
mod management;
mod server;
mod pid;

use crate::backup::BackupManager;
use crate::cli::Cli;
use crate::config::Config;
use crate::management::RconService;
use crate::management::rcon::rcon_server::RconServer;
use clap::Parser;
use fork::{Fork, daemon};
use std::process::Command;
use tonic::transport::Server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.daemon {
        if let Ok(Fork::Child) = daemon(false, false) {
            Command::new("bedrockd").spawn().expect("failed to execute process");
        }
        Ok(())
    } else {
        // Attempt to obtain pid lock file
        let _lock_handle = pid::lock_pid_file()?;

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(run())?;
        Ok(())
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running bedrockd in non-daemonic mode");
    // Parse config file in /etc/bedrockd.conf
    let config = Config::open()?;

    let backup_manager = BackupManager::new(config.backup_frequency, config.backup_dir.into());

    let addr = "[::1]:10000".parse().unwrap();

    let rcon = RconService {};

    let svc = RconServer::new(rcon);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}