use std::{io, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStderr, ChildStdin, ChildStdout, Command},
    spawn,
    sync::{
        broadcast::{Receiver, Sender},
        mpsc,
    },
    task::JoinHandle,
};
use tonic::Status;

use crate::management::rcon::{ServerStdioRequest, ServerStdioResponse};

pub struct Wrapper {
    // pub mc_server_process: Command,
    stdin_task: JoinHandle<()>,
    stdout_task: JoinHandle<()>,
    stderr: Option<ChildStderr>,
    rx: Receiver<Result<ServerStdioResponse, Status>>,
    tx: mpsc::Sender<ServerStdioRequest>,
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

        spawn(async move {
            let status = child
                .wait()
                .await
                .expect("Server process encountered an error");
            println!("Server status was: {}", status);
        });
        // Broadcast channel for stdout
        let (tx, rx) = tokio::sync::broadcast::channel::<Result<ServerStdioResponse, Status>>(4);

        // MPSC channel for stdin
        let (tx_in, mut rx_in) = tokio::sync::mpsc::channel::<ServerStdioRequest>(32);

        // spawn the reader task
        let tx_out = tx.clone();
        let stdout_task = spawn(async move {
            let stdout_handle = stdout.unwrap();
            let mut reader = BufReader::new(stdout_handle);
            while let Ok(Some(line)) = Self::next_line(&mut reader).await {
                // ignore errors if no one is listening
                let result = ServerStdioResponse {
                    is_error: false,
                    output: line,
                };
                let _ = tx_out.send(Ok(result));
            }
        });

        let stdin_task = spawn(async move {
            let mut stdin_handle = stdin.unwrap();
            while let Some(request) = rx_in.recv().await {
                let result = Self::send_line(&mut stdin_handle, &request.command.as_str()).await;
                match result {
                    Ok(_) => (),
                    Err(_) => {
                        let _ = tx.send(Err(Status::unknown(format!(
                            "Unable to send command: {}",
                            request.command
                        ))));
                    }
                }
            }
        });

        Self {
            stdin_task,
            stdout_task,
            stderr,
            rx,
            tx: tx_in,
        }
    }

    async fn send_line(stdin: &mut ChildStdin, line: &str) -> io::Result<()> {
        stdin.write_all(line.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await
    }

    pub async fn next_line(reader: &mut BufReader<ChildStdout>) -> io::Result<Option<String>> {
        let mut line = String::new();
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

    pub fn stdout_subscribe(&self) -> Receiver<Result<ServerStdioResponse, Status>> {
        self.rx.resubscribe()
    }

    pub fn get_stdin(&self) -> mpsc::Sender<ServerStdioRequest> {
        self.tx.clone()
    }
}
