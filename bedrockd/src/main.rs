mod backup;
mod cli;
mod config;
mod management;
mod pid;
mod server;
pub mod wrapper;

use crate::backup::BackupManager;
use crate::cli::Cli;
use crate::config::Config;
use crate::management::Rcon;
use crate::management::rcon::rcon_service_server::RconServiceServer;
use clap::Parser;
use fork::{Fork, daemon};
use std::process::Command;
use tonic::transport::Server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.daemon {
        if let Ok(Fork::Child) = daemon(false, false) {
            Command::new("bedrockd")
                .spawn()
                .expect("failed to execute process");
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

    if config.gRPC.enabled {
        let addr = format!("[::1]:{}", config.gRPC.port).parse().unwrap();

        let rcon = Rcon {};

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(management::FILE_DESCRIPTOR_SET)
            .build_v1alpha()
            .unwrap();

        let svc = RconServiceServer::new(rcon);

        Server::builder()
            .add_service(reflection_service)
            .add_service(svc)
            .serve(addr)
            .await?;
    }

    Ok(())
}
