use crate::management::rcon::ServerStdioResponse;
use crate::wrapper::Wrapper;
use std::path::PathBuf;
use std::process::{ChildStdin, ChildStdout, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::Sender;
use tonic::{Code, Status};
// pub fn backup(iostream: Stdio)

pub struct BackupManager {
    pub frequency: u16,
    pub dir: PathBuf,
    pub wrapper: Wrapper,
}

impl BackupManager {
    pub fn new(frequency: u16, dir: PathBuf, wrapper: Wrapper) -> BackupManager {
        Self {
            frequency,
            dir,
            wrapper,
        }
    }

    pub async fn initiate_backup(
        &mut self,
        tx: Sender<ServerStdioResponse>,
    ) -> Result<ServerStdioResponse, Status> {
        // self.wrapper.send_line("line").await?;

        // if let Some(mut stdin) = self.stdin.take() {
        //     let result = tokio::task::spawn(async move {
        //         stdin
        //             .write_all("".as_bytes())
        //             .expect("Unable to write bytes to stdin!")
        //     })
        //     .await
        //     .or_else(|e| Err(Status::new(Code::Internal, e.to_string())));
        // } else {
        // };
        todo!()
    }
}
