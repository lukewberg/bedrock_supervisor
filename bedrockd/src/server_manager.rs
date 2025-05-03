use crate::management::rcon::{ServerStdioRequest, ServerStdioResponse};
use crate::wrapper::Wrapper;
use flate2::write::GzEncoder;
use flate2::Compression;
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tar::Header;
use tokio::spawn;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tonic::Status;
use crate::config::Backup;
// pub fn backup(iostream: Stdio)

pub struct ServerManager {
    pub backup_config: Backup,
    pub wrapper: Wrapper,
}

pub struct BackupOutput {
    pub path: String,
    pub size: usize,
}

impl ServerManager {
    pub fn new(backup_config: Backup, wrapper: Wrapper) -> ServerManager {
        Self {
            backup_config,
            wrapper,
        }
    }

    pub async fn initiate_backup(&mut self) -> Result<ServerStdioResponse, Status> {
        // self.wrapper.send_line("line").await?;
        // say §g§l[bedrockd]§r Starting server backup...
        //
        let stdin = self.wrapper.get_stdin();
        if let Ok(()) = stdin
            .send("say §g§l[bedrockd]§r Starting server backup...".into())
            .await
        {}
        todo!()
    }

    pub fn spawn_backup_task(&self) -> JoinHandle<()> {
        let backup_regex = Regex::new(
            // TODO: Interpolate level name into regex string
            format!(
                r#"(?m)(?P<path>{}/[^\s:,]+):(?P<size>\d+)\s*,?\s*"#,
                "Bedrock level"
            )
            .as_str(),
        )
        .unwrap();
        let stdout = self.wrapper.stdout_subscribe();
        let stdin = self.wrapper.get_stdin();
        let backup_dir = self.dir.clone();
        spawn(Self::parse_stdout(stdin, stdout, backup_dir, backup_regex))
    }

    pub async fn spawn_scheduled_backup_task(&self) -> JoinHandle<()> {
        let stdin = self.wrapper.get_stdin();
        spawn(async move {
            let mut backup_interval = interval(Duration::from_secs(300));
            let mut cmd_interval = interval(Duration::from_secs(5));
            cmd_interval.tick().await;
            backup_interval.tick().await; // Wait for the first interval
            loop {
                backup_interval.tick().await;
                stdin.send("save hold".into()).await.unwrap();
                cmd_interval.tick().await;
                stdin.send("save query".into()).await.unwrap();
            }
        })
    }

    async fn parse_stdout(
        mut stdin: mpsc::Sender<ServerStdioRequest>,
        mut stdout: Receiver<Result<ServerStdioResponse, Status>>,
        backup_dir: PathBuf,
        backup_regex: Regex,
    ) {
        loop {
            match stdout.recv().await {
                Ok(Ok(output)) => {
                    let message = output.output;
                    if backup_regex.is_match(message.as_str()) {
                        println!("{message}");
                        let mut files: Vec<BackupOutput> = Vec::new();
                        backup_regex
                            .captures_iter(message.as_str())
                            .for_each(|captures| {
                                let (_, [path, size]) = captures.extract();
                                let full_path = format!("./worlds/{}", path);
                                files.push(BackupOutput {
                                    path: full_path,
                                    size: size.parse().unwrap(),
                                });
                            });
                        Self::create_archive(&backup_dir, files).unwrap();
                        stdin.send("save resume".into()).await.unwrap();
                    }
                }

                // your upstream sent back an error
                Ok(Err(status)) => {
                    eprintln!("upstream error: {:?}", status);
                }

                // you fell behind; skip those messages
                Err(RecvError::Lagged(skipped)) => {
                    eprintln!("skipped {} lines", skipped);
                }

                // the sender(s) have all been dropped → nothing more to read
                Err(RecvError::Closed) => {
                    println!("stdout channel closed, ending parser");
                    break;
                }
            }
        }
        // disconnect regex: "Player disconnected: (?P<username>\w+[^,])"gm
        // backup files and sizes: "(\w+ ?\w+/\w*/*\w*-*\.*\w*:(\d*))"gm
        // "(?P<path>\w+ ?\w+/\w*/*\w*-*\.*\w*):(?P<size>\d*)"gm
    }

    fn create_archive(dir: &PathBuf, files: Vec<BackupOutput>) -> io::Result<()> {
        // let tar_gz = File::create(dir.join("archive.tar.gz"))?;
        println!(
            "Backup file: {}",
            dir.join("archive.tar.gz").to_str().unwrap()
        );
        let tar_gz = OpenOptions::new()
            .write(true)
            .create(true)
            .open(dir.join("archive.tar.gz"))?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);

        files.iter().for_each(|file| {
            println!("Reading backup file: {}", file.path);
            let path = PathBuf::from(&file.path);
            let prefix = Path::new("./worlds");
            let stripped = path.strip_prefix(prefix).unwrap();
            let archive_path = PathBuf::from(".").join(stripped);

            let mut data: Vec<u8> = vec![0u8; file.size];
            println!("Opening: {}", path.display());
            let mut f = File::open(&path).unwrap();

            f.read_exact(&mut data).unwrap();

            let mut header = Header::new_gnu();
            header.set_size(file.size as u64);
            header.set_cksum();
            tar.append_data(&mut header, &archive_path, data.as_slice())
                .unwrap();
        });
        tar.finish()
    }
}
