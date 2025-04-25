use crate::management::rcon::ServerStdioResponse;
use crate::wrapper::Wrapper;
use std::path::PathBuf;
use std::process::{ChildStdin, ChildStdout, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::spawn;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tonic::{Code, Status};
// pub fn backup(iostream: Stdio)

pub struct ServerManager {
    pub frequency: u16,
    pub dir: PathBuf,
    pub wrapper: Wrapper,
}

impl ServerManager {
    pub fn new(frequency: u16, dir: PathBuf, wrapper: Wrapper) -> ServerManager {
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
        self.wrapper.send_line("line").await?;

        todo!()
    }

    pub async fn stream_stdout(
        &mut self,
        tx: Sender<Result<ServerStdioResponse, Status>>,
    ) -> Result<ServerStdioResponse, Status> {
        //     Continuously await the next line and send it
        loop {
            let line = self.wrapper.next_line().await?;
            if let Some(line) = line {
                let response = ServerStdioResponse {
                    is_error: false,
                    output: line,
                };
                if let Err(status) = tx.try_send(Ok(response)) {
                    return Err(Status::internal(format!("{}", status)));
                }
            }
        }
    }
    
    pub async fn spawn_stdout_task(&mut self, tx: Sender<Result<ServerStdioResponse, Status>>) -> JoinHandle<()> {
        let stdout = self.wrapper.sts;
         spawn(async move {
            self.wrapper.next_line().await.unwrap();
        })
    }
}
