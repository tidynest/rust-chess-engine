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

/// Run the CLI on `input` to its end and return what it printed.
fn run(input: &str) -> std::io::Result<String> {
    let mut cli = Command::new(env!("CARGO_BIN_EXE_chess-cli"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = cli.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }
    Ok(String::from_utf8_lossy(&cli.wait_with_output()?.stdout).into_owned())
}

#[test]
fn a_finished_game_can_still_be_printed_or_taken_back() {
    let out = run("f3\ne5\ng4\nQh4\npgn\nundo\nQe7\npgn\n").unwrap();
    assert!(out.contains("1. f3 e5 2. g4 Qh4# 0-1"), "{out}");
    assert!(out.contains("1. f3 e5 2. g4 Qe7 *"), "{out}");

    // Bare kings: a draw, though the kings still have moves.
    let out = run("fen 8/8/8/8/8/8/8/K6k w - - 0 1\nKb1\nfen\n").unwrap();
    assert!(out.contains("The game is over"), "{out}");
    assert!(out.contains("8/8/8/8/8/8/8/K6k w - - 0 1"), "{out}");
}
