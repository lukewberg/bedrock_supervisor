use fs2::FileExt;
use std::fs::{File, OpenOptions};
pub fn lock_pid_file() -> std::io::Result<File> {
    let pid_handle = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("/var/run/bedrockd.pid")?;

    pid_handle.try_lock_exclusive()?;
    Ok(pid_handle)
}
