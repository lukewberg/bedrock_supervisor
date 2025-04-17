use std::{io, path::PathBuf, process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio}};

use tokio::sync::mpsc::Sender;
use tonic::Status;

use crate::management::rcon::ServerStdioResponse;

pub struct Wrapper {
    // pub mc_server_process: Command,
    child: Child,
}

impl Wrapper {
    pub fn new(bin_path: PathBuf) -> Self {
        let mut child = Command::new("bedrock_server")
            .current_dir(bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("unable to start bedrock server!");

        Self { child }
    }

    pub fn get_stdin(&mut self) -> Option<&mut ChildStdin> {
        self.child.stdin.as_mut()
    }

    pub fn get_stdout(&mut self) -> Option<&mut ChildStdout> {
        self.child.stdout.as_mut()
    }

    pub fn get_stderr(&mut self) -> Option<&mut ChildStderr> {
        self.child.stderr.as_mut()
    }
}
