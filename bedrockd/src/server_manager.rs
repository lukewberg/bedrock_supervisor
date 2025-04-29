use crate::management::rcon::{ServerStdioRequest, ServerStdioResponse};
use crate::wrapper::Wrapper;
use std::path::PathBuf;
use tokio::spawn;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tonic::Status;
// pub fn backup(iostream: Stdio)

pub struct ServerManager {
    pub frequency: u16,
    pub dir: PathBuf,
    pub wrapper: Wrapper,
    stdout_parser: JoinHandle<()>,
}

impl ServerManager {
    pub fn new(frequency: u16, dir: PathBuf, wrapper: Wrapper) -> ServerManager {
        let stdout_parser = spawn(Self::parse_stdout());
        Self {
            frequency,
            dir,
            wrapper,
            stdout_parser,
        }
    }

    pub async fn initiate_backup(&mut self) -> Result<ServerStdioResponse, Status> {
        // self.wrapper.send_line("line").await?;
        // say §g§l[bedrockd]§r Starting server backup...
        //
        let stdin = self.wrapper.get_stdin();
        if let Ok(()) = stdin
            .send(ServerStdioRequest {
                command: "say §g§l[bedrockd]§r Starting server backup...".into(),
            })
            .await
        {}
        todo!()
    }

    async fn parse_stdout() {
        // disconnect regex: "Player disconnected: (?P<username>\w+[^,])"gm
        // backup files and sizes: "(\w+ ?\w+/\w*/*\w*-*\.*\w*:(\d*))"gm
    }
}
