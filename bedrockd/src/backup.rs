use crate::management::rcon::ServerStdioResponse;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::sync::mpsc::Sender;
use tonic::Status;
// pub fn backup(iostream: Stdio)

pub struct BackupManager {
    pub frequency: u16,
    pub dir: PathBuf,
}

impl BackupManager {
    pub fn new(frequency: u16, dir: PathBuf) -> BackupManager {
        Self { frequency, dir }
    }

    pub async fn initiate_backup(
        &self,
        tx: Sender<ServerStdioResponse>,
    ) -> Result<ServerStdioResponse, Status> {
        todo!()
    }
}
