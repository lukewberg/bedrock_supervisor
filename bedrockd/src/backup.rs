use std::path::PathBuf;
use std::process::Stdio;

// pub fn backup(iostream: Stdio)

pub struct BackupManager {
    pub frequency: u16,
    pub dir: PathBuf,
}

impl BackupManager {
    pub fn new(frequency: u16, dir: PathBuf) -> BackupManager {
        Self {
            frequency,
            dir
        }
    }
}