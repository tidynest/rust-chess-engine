//! The `chess-cli` binary driven through its standard input.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn cli_exits_when_its_input_ends() {
    let mut cli = Command::new(env!("CARGO_BIN_EXE_chess-cli"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    // No `quit`: dropping the pipe is the end of input, as Ctrl+D is.
    cli.stdin.take().unwrap().write_all(b"e4\n").unwrap();

    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = cli.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            cli.kill().unwrap();
            panic!("chess-cli kept running after its input ended");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(status.success());
}
