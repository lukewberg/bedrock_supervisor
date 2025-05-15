use crate::config::{Backup, BackupFrequency, BackupSchedule};
use crate::management::rcon::{ServerStdioRequest, ServerStdioResponse};
use chrono::{TimeDelta, Timelike, Utc};
use flate2::Compression;
use flate2::write::GzEncoder;
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tar::{Builder, Header};
use tokio::spawn;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};
use tonic::Status;
use uuid::Uuid;

pub struct BackupManager {
    backup_config: Backup,
    stdout: Receiver<Result<ServerStdioResponse, Status>>,
    stdin: Sender<ServerStdioRequest>,
    regex: Regex,
}

pub struct BackupOutput {
    pub path: String,
    pub size: usize,
}

impl BackupManager {
    pub fn new(
        backup_config: Backup,
        stdout: Receiver<Result<ServerStdioResponse, Status>>,
        stdin: Sender<ServerStdioRequest>,
    ) -> Self {
        let regex = Regex::new(
            format!(
                r#"(?m)(?P<path>{}/[^\s:,]+):(?P<size>\d+)\s*,?\s*"#,
                backup_config.level_name
            )
            .as_str(),
        )
        .unwrap();

        Self {
            backup_config,
            stdout,
            stdin,
            regex,
        }
    }

    pub fn create_scheduled_tasks(&mut self) -> () {
        let schedules: Vec<_> = self.backup_config.schedule.drain(0..).collect();
        for schedule in schedules {
            if !schedule.enabled {
                return;
            }
            self.spawn_scheduled_backup_task(schedule);
        }
    }

    pub fn spawn_scheduled_backup_task(
        &self,
        schedule: BackupSchedule,
    ) -> (JoinHandle<()>, JoinHandle<()>) {
        let stdin = self.stdin.clone();
        let save_handle = spawn(async move {
            // let mut backup_interval = interval(Duration::from_secs(300));
            let mut cmd_interval = interval(Duration::from_secs(5));
            println!("Setup intervals");
            cmd_interval.tick().await; // Wait for the first interval
            loop {
                Self::scheduled_wait(&schedule).await;
                stdin
                    .send("say §g§l[bedrockd]§r Starting server backup...".into())
                    .await
                    .unwrap();
                stdin.send("save hold".into()).await.unwrap();
                cmd_interval.tick().await;
                stdin.send("save query".into()).await.unwrap();
            }
        });
        let parse_handle = spawn(Self::parse_stdout(
            self.backup_config.clone(),
            self.stdin.clone(),
            self.stdout.resubscribe(),
            self.regex.clone(),
        ));
        (save_handle, parse_handle)
    }

    async fn parse_stdout(
        backup_config: Backup,
        stdin: Sender<ServerStdioRequest>,
        mut stdout: Receiver<Result<ServerStdioResponse, Status>>,
        regex: Regex,
    ) {
        loop {
            match stdout.recv().await {
                Ok(Ok(output)) => {
                    let message = output.output;
                    if regex.is_match(message.as_str()) {
                        println!("{message}");
                        let mut files: Vec<BackupOutput> = Vec::new();
                        regex.captures_iter(message.as_str()).for_each(|captures| {
                            let (_, [path, size]) = captures.extract();
                            let full_path = format!("./worlds/{}", path);
                            files.push(BackupOutput {
                                path: full_path,
                                size: size.parse().unwrap(),
                            });
                        });
                        Self::backup(&backup_config, files).unwrap();
                        stdin.send("save resume".into()).await.unwrap();
                        stdin
                            .send("say §g§l[bedrockd]§r Server backup complete!".into())
                            .await
                            .unwrap();
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

    async fn scheduled_wait(schedule: &BackupSchedule) {
        match schedule.frequency {
            BackupFrequency::MINUTE => {
                let mut interval = interval(Duration::from_secs(60 * (schedule.value as u64)));
                interval.tick().await;
                interval.tick().await;
            }
            BackupFrequency::HOURLY => {
                let mut interval = interval(Duration::from_secs(3600 * (schedule.value as u64)));
                interval.tick().await;
                interval.tick().await;
            }
            BackupFrequency::DAILY => {
                let now = Utc::now();
                let hour = (schedule.value / 100) as u32;
                let minute = (schedule.value % 100) as u32;
                // Calculate next run time
                // let scheduled_time = NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
                let scheduled_time = now
                    .with_hour(hour)
                    .unwrap()
                    .with_minute(minute)
                    .unwrap()
                    .with_second(0)
                    .unwrap();
                if now <= scheduled_time {
                    // Calculate time to next
                    let next = scheduled_time - now;
                    let time_until = (next).num_seconds();
                    sleep(Duration::from_secs(time_until as u64)).await;
                } else {
                    let next = scheduled_time + TimeDelta::days(1);
                    let time_until = (next - now).num_seconds();
                    sleep(Duration::from_secs(time_until as u64)).await;
                }
                // let mut next = Utc::now().time().add
            }
            BackupFrequency::WEEKLY => {
                todo!("Weekly schedule not yet implemented!")
            }
        }
    }

    fn check_path(&self) -> bool {
        Path::new(&self.backup_config.path).exists()
    }

    fn backup(backup_config: &Backup, files: Vec<BackupOutput>) -> io::Result<()> {
        let archive = Self::build_archive(backup_config.path.clone())?;
        Self::append_to_archive(archive, files)?;
        Ok(())
    }

    fn open_archive(path: String) -> io::Result<File> {
        let dir = Path::new(path.as_str());
        let id = Uuid::new_v4();
        let archive_name = format!("archive-{}.tar.gz", id);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(dir.join(archive_name))?;
        Ok(file)
    }

    fn build_archive(path: String) -> io::Result<Builder<GzEncoder<File>>> {
        let tar_gz = Self::open_archive(path)?;
        let enc = GzEncoder::new(tar_gz, Compression::best());
        Ok(tar::Builder::new(enc))
    }

    fn append_to_archive(
        mut archive: Builder<GzEncoder<File>>,
        files: Vec<BackupOutput>,
    ) -> io::Result<GzEncoder<File>> {
        files.iter().for_each(|file| {
            let path = PathBuf::from(&file.path);
            let prefix = Path::new("./worlds");
            let stripped = path.strip_prefix(prefix).unwrap();
            let archive_path = PathBuf::from(".").join(stripped);

            let mut data: Vec<u8> = vec![0u8; file.size];
            let mut f = File::open(&path).unwrap();
            let metadata = f.metadata().unwrap();

            f.read_exact(&mut data).unwrap();

            let mut header = Header::new_gnu();
            header.set_metadata(&metadata);
            header.set_size(file.size as u64);
            header.set_cksum();
            archive
                .append_data(&mut header, &archive_path, data.as_slice())
                .unwrap();
        });
        archive.into_inner()
    }
}
