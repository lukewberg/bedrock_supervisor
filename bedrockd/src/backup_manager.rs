use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tonic::Status;
use crate::config::Backup;
use crate::management::rcon::{ServerStdioRequest, ServerStdioResponse};

pub struct BackupManager {
    backup_config: Backup,
    stdout: Receiver<Result<ServerStdioResponse, Status>>,
    stdin: Sender<ServerStdioRequest>,
}