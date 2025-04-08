use std::io;
use std::process::{Child, Command, Stdio};

pub fn start_server() -> io::Result<Child> {
    Command::new("bedrock_server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
}
