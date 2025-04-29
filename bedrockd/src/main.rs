mod cli;
mod config;
mod management;
mod pid;
mod server;
mod server_manager;
pub mod wrapper;

use crate::cli::Cli;
use crate::config::Config;
use crate::management::rcon::rcon_service_server::RconServiceServer;
use crate::management::Rcon;
use crate::server_manager::ServerManager;
use clap::Parser;
use fork::{daemon, Fork};
use std::process::Command;
use tonic::transport::Server;
use wrapper::Wrapper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.config {
        Config::create().expect("Failed to create config");
        return Ok(());
    }

    if cli.daemon {
        println!("Running bedrockd in daemonic mode");
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

    let wrapper = Wrapper::new(config.server.path.clone().as_str());
    let server_manager =
        ServerManager::new(config.backup_frequency, config.backup_dir.into(), wrapper);

    if config.gRPC.enabled {
        let addr = format!("0.0.0.0:{}", config.gRPC.port).parse().unwrap();

        let rcon = Rcon::new(server_manager);

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
