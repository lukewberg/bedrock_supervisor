use crate::config::{Backup, BackupFrequency, BackupSchedule};
use crate::management::rcon::{ServerStdioRequest, ServerStdioResponse};
use chrono::{Datelike, TimeDelta, Timelike, Utc};
use flate2::Compression;
use flate2::write::GzEncoder;
use regex::Regex;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tar::{Builder, Header};
use tokio::spawn;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tonic::Status;
use uuid::Uuid;

pub struct BackupManager {
    backup_config: Backup,
    stdout: Receiver<Result<ServerStdioResponse, Status>>,
    stdin: Sender<ServerStdioRequest>,
    regex: Regex,
    players_online: Arc<Mutex<HashSet<String>>>,
}

pub struct BackupOutput {
    pub path: String,
    pub size: usize,
}

pub struct ArchiveInfo {
    pub id: String,
    pub modified: std::time::SystemTime,
    pub size: u64,
}

impl BackupManager {
    pub fn new(
        backup_config: Backup,
        stdout: Receiver<Result<ServerStdioResponse, Status>>,
        stdin: Sender<ServerStdioRequest>,
    ) -> Self {
        let regex = Regex::new(
            format!(
                r#"(?P<path>{}/[^\s:,]+):(?P<size>\d+)\s*,?\s*"#,
                regex::escape(&backup_config.level_name)
            )
            .as_str(),
        )
        .unwrap();

        Self {
            backup_config,
            stdout,
            stdin,
            regex,
            players_online: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn list_backups(&self, frequency: &BackupFrequency) -> io::Result<Vec<ArchiveInfo>> {
        let subfolder = match frequency {
            BackupFrequency::Minute => "minute",
            BackupFrequency::Hourly => "hourly",
            BackupFrequency::Daily => "daily",
            BackupFrequency::Weekly => "weekly",
        };
        let path = format!("{}/{}", self.backup_config.path, subfolder);
        let dir = Path::new(&path);
        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut archives: Vec<ArchiveInfo> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or_else(|| false, |ext| ext == "gz")
            })
            .filter_map(|e| {
                let meta = e.metadata().ok()?;
                let modified = meta.modified().ok()?;
                let size = meta.len();
                let filename = e.file_name();
                let name = filename.to_string_lossy();
                let id = name
                    .strip_prefix("archive-")?
                    .strip_suffix(".tar.gz")?
                    .to_string();
                Some(ArchiveInfo { id, modified, size })
            })
            .collect();

        archives.sort_by_key(|a| a.modified);
        Ok(archives)
    }

    pub fn create_scheduled_tasks(&mut self) {
        self.spawn_player_tracker();
        let schedules: Vec<_> = self.backup_config.schedule.drain(0..).collect();
        for schedule in schedules {
            if !schedule.enabled {
                continue;
            }
            self.spawn_scheduled_backup_task(schedule);
        }
    }

    fn spawn_player_tracker(&self) -> JoinHandle<()> {
        let mut stdout = self.stdout.resubscribe();
        let players = self.players_online.clone();

        spawn(async move {
            loop {
                match stdout.recv().await {
                    Ok(Ok(output)) => {
                        let line = &output.output;
                        if (line.contains("Player connected:")
                            || line.contains("Player disconnected:"))
                            && let Some(xuid) = Self::parse_xuid(line)
                        {
                            let mut set = players.lock().unwrap();
                            if line.contains("Player connected:") {
                                println!("Player joined, xuid: {xuid}");
                                set.insert(xuid);
                            } else {
                                println!("Player left, xuid: {xuid}");
                                set.remove(&xuid);
                            }
                            println!("Players online: {}", set.len());
                        }
                    }
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(n)) => eprintln!("player tracker skipped {n} lines"),
                    _ => {}
                }
            }
        })
    }

    fn parse_xuid(line: &str) -> Option<String> {
        let start = line.find("xuid: ")? + 6;
        let rest = &line[start..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }

    pub fn spawn_scheduled_backup_task(&self, schedule: BackupSchedule) -> JoinHandle<()> {
        let stdin = self.stdin.clone();
        let mut stdout = self.stdout.resubscribe();
        let backup_config = self.backup_config.clone();
        let regex = self.regex.clone();
        let players_online = self.players_online.clone();

        spawn(async move {
            loop {
                Self::scheduled_wait(&schedule).await;

                if players_online.lock().unwrap().is_empty() {
                    println!("No players online, skipping backup");
                    continue;
                }

                stdin
                    .send("say §g§l[bedrockd]§r Starting server backup...".into())
                    .await
                    .unwrap();
                stdin.send("save hold".into()).await.unwrap();

                // Poll with save query until the server reports files are ready.
                // "Data saved..." and the file list arrive as two consecutive lines
                // in response to save query — we match on the file list directly.
                let mut files: Vec<BackupOutput> = 'query: loop {
                    sleep(Duration::from_secs(3)).await;
                    stdin.send("save query".into()).await.unwrap();

                    loop {
                        match stdout.recv().await {
                            Ok(Ok(output)) => {
                                if regex.is_match(&output.output) {
                                    let files = regex
                                        .captures_iter(&output.output)
                                        .map(|c| {
                                            let (_, [path, size]) = c.extract();
                                            BackupOutput {
                                                path: format!("./worlds/{path}"),
                                                size: size.parse().unwrap(),
                                            }
                                        })
                                        .collect();
                                    break 'query files;
                                }
                                // "not completed" → stop reading this burst, re-poll
                                if output
                                    .output
                                    .contains("A previous save has not been completed")
                                {
                                    break;
                                }
                                // any other line: keep reading
                            }
                            Err(RecvError::Closed) => return,
                            Err(RecvError::Lagged(n)) => eprintln!("skipped {n} lines"),
                            Ok(Err(e)) => eprintln!("upstream error: {e:?}"),
                        }
                    }
                };

                // save query only lists mutable files (CURRENT, MANIFEST, .log, level.dat).
                // .ldb SSTables are immutable and safe to copy while save hold pauses compaction.
                let db_path = format!("./worlds/{}/db", backup_config.level_name);
                if let Ok(entries) = std::fs::read_dir(&db_path) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.extension().map_or(false, |ext| ext == "ldb") {
                            if let Ok(meta) = entry.metadata() {
                                files.push(BackupOutput {
                                    path: path.to_string_lossy().into_owned(),
                                    size: meta.len() as usize,
                                });
                            }
                        }
                    }
                }

                let subfolder = match schedule.frequency {
                    BackupFrequency::Minute => "minute",
                    BackupFrequency::Hourly => "hourly",
                    BackupFrequency::Daily => "daily",
                    BackupFrequency::Weekly => "weekly",
                };
                let backup_path = format!("{}/{}", backup_config.path, subfolder);

                match Self::backup(&backup_path, schedule.limit, files) {
                    Ok(_) => {
                        stdin.send("save resume".into()).await.unwrap();
                        stdin
                            .send("say §g§l[bedrockd]§r Server backup complete!".into())
                            .await
                            .unwrap();
                    }
                    Err(e) => {
                        eprintln!("Backup failed: {e}");
                        stdin.send("save resume".into()).await.unwrap();
                        stdin
                            .send("say §c§l[bedrockd]§r Server backup failed!".into())
                            .await
                            .unwrap();
                    }
                }
            }
        })
    }

    async fn scheduled_wait(schedule: &BackupSchedule) {
        match schedule.frequency {
            BackupFrequency::Minute => {
                sleep(Duration::from_secs(60 * schedule.value as u64)).await;
            }
            BackupFrequency::Hourly => {
                sleep(Duration::from_secs(3600 * schedule.value as u64)).await;
            }
            BackupFrequency::Daily => {
                let now = Utc::now();
                let hour = (schedule.value / 100) as u32;
                let minute = (schedule.value % 100) as u32;
                let scheduled_time = now
                    .with_hour(hour)
                    .unwrap()
                    .with_minute(minute)
                    .unwrap()
                    .with_second(0)
                    .unwrap();
                let next = if now <= scheduled_time {
                    scheduled_time
                } else {
                    scheduled_time + TimeDelta::days(1)
                };
                let secs = (next - now).num_seconds().max(0) as u64;
                sleep(Duration::from_secs(secs)).await;
            }
            BackupFrequency::Weekly => {
                let now = Utc::now();
                let current_weekday = now.weekday().num_days_from_monday();
                let target_weekday = schedule.value as u32;
                let days_until = match (target_weekday + 7 - current_weekday) % 7 {
                    0 => 7,
                    d => d,
                };
                let secs = TimeDelta::days(days_until as i64).num_seconds() as u64;
                sleep(Duration::from_secs(secs)).await;
            }
        }
    }

    fn backup(path: &str, limit: u16, files: Vec<BackupOutput>) -> io::Result<()> {
        std::fs::create_dir_all(path)?;
        let archive = Self::build_archive(path.to_string())?;
        Self::append_to_archive(archive, files)?;
        Self::prune_archives(path, limit)?;
        Ok(())
    }

    fn prune_archives(path: &str, limit: u16) -> io::Result<()> {
        if limit == 0 {
            return Ok(());
        }

        let mut archives: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or_else(|| false, |ext| ext == "gz")
            })
            .filter_map(|e| {
                let modified = e.metadata().ok()?.modified().ok()?;
                Some((modified, e.path()))
            })
            .collect();

        // Sort oldest first so we remove from the front
        archives.sort_by_key(|(modified, _)| *modified);

        let excess = archives.len().saturating_sub(limit as usize);
        for (_, path) in archives.iter().take(excess) {
            if let Err(e) = std::fs::remove_file(path) {
                eprintln!("Failed to remove old backup {}: {e}", path.display());
            }
        }

        Ok(())
    }

    fn open_archive(path: String) -> io::Result<File> {
        let dir = Path::new(&path);
        let archive_name = format!("archive-{}.tar.gz", Uuid::new_v4());
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
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
