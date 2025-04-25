use std::{io, path::PathBuf, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStderr, ChildStdin, ChildStdout, Command},
};

pub struct Wrapper {
    // pub mc_server_process: Command,
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
}

impl Wrapper {
    pub fn new(bin_path: &str) -> Self {
        let mut child = Command::new(format!("{}/bedrock_server", bin_path))
            .current_dir(bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .expect("unable to start bedrock server!");

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .expect("Server process encountered an error");
            println!("Server status was: {}", status);
        });

        Self {
            stdin,
            stdout,
            stderr,
        }
    }

    pub async fn send_line(&mut self, line: &str) -> io::Result<()> {
        let stdin = self.stdin.as_mut().unwrap();
        stdin.write_all(line.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await
    }

    pub async fn next_line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        let stdout = self.stdout.as_mut().unwrap();
        let mut reader = BufReader::new(stdout);
        let num_read = reader.read_line(&mut line).await?;
        if num_read == 0 {
            Ok(None)
        } else {
            Ok(Some(line))
        }
    }

    pub fn get_stderr(&mut self) -> Option<&mut ChildStderr> {
        self.stderr.as_mut()
    }
}
