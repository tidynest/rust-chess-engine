// Stockfish UCI engine wrapper using tokio for async I/O
// This module handles all communication with the Stockfish chess engine

use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

/// Commands that can be sent to the engine
#[derive(Debug, Clone)]
pub enum EngineCommand {
    /// Request best move for a position with configurable settings
    GetBestMove {
        fen: String,
        depth: Option<u32>,
        movetime: Option<u64>,
        skill_level: i32,
    },
    /// Quit the engine
    Quit,
}

/// Response received from the engine
#[derive(Debug, Clone)]
pub enum EngineResponse {
    /// Engine is ready
    Ready,
    /// Search information update
    Info {
        depth: u32,
        score: i32,         // Centipawns (100 = 1 pawn advantage)
        nodes: u64,
        nps: u64,           // Node per second
        pv: Vec<String>,    // Principal variation (best line)
    },
    /// Engine found the best move
    BestMove {
        mv: String,
        ponder: Option<String>,
    },
    /// Error or unrecognised message
    Error(String),
}

/// Stockfish engine wrapper with async communication
pub struct StockfishEngine {
    child: Child,
    stdin_tx: mpsc::UnboundedSender<String>,
    stdout_rx: mpsc::UnboundedReceiver<String>,
}

impl StockfishEngine {
    /// Create a new Stockfish engine instance
    pub async fn new(path: &str) -> Result<Self> {
        // Spawn the Stockfish process
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)  // Ensure cleanup
            .spawn()
            .context("Failed to spawn stockfish process")?;

        let mut stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        // Create channels for async communication
        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<String>();
        let (stdout_tx, stdout_rx) = mpsc::unbounded_channel::<String>();

        // Spawn writer task - writes commands to stdin
        tokio::spawn(async move {
            while let Some(line) = stdin_rx.recv().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        // Spawn reader task - reads responses from stdout
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if stdout_tx.send(line).is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            child,
            stdin_tx,
            stdout_rx,
        })
    }

    /// Initialise the engine and wait for it to be ready
    pub async fn initialise(&mut self) -> Result<()> {
        // Send UCI command
        self.send_command("uci").await?;

        // Wait for uciok
        timeout(Duration::from_secs(5), async {
            while let Some(line) = self.stdout_rx.recv().await {
                if line == "uciok" {
                    return Ok(());
                }
            }
            Err(anyhow::anyhow!("Engine did not respond with uciok"))
        })
        .await
        .context("Timeout waiting for engine initialisation")??;

        // Configure engine options
        self.send_command("setoption name Hash value 128").await?;
        self.send_command("setoption name Threads value 4").await?;

        // Wait for ready
        self.wait_ready().await?;

        Ok(())
    }

    /// Send a raw command to the engine
    pub async fn send_command(&self, cmd: &str) -> Result<()> {
        self.stdin_tx
            .send(format!("{}\n", cmd))
            .context("Failed to send command to engine")?;
        Ok(())
    }

    /// Wait for the engine to be ready
    pub async fn wait_ready(&mut self) -> Result<()> {
        self.send_command("isready").await?;

        timeout(Duration::from_secs(5), async {
            while let Some(line) = self.stdout_rx.recv().await {
                if line == "readyok" {
                    return Ok(());
                }
            }
            Err(anyhow::anyhow!("Engine did not respond with readyok"))
        })
        .await
        .context("Timeout waiting for engine ready")??;

        Ok(())
    }

    /// Start a new game
    pub async fn new_game(&mut self) -> Result<()> {
        self.send_command("ucinewgame").await?;
        self.wait_ready().await?;
        Ok(())
    }

    /// Set the board position
    pub async fn set_position(&mut self, position_cmd: &str) -> Result<()> {
        self.send_command(&format!("position {}", position_cmd)).await?;
        Ok(())
    }

    /// Start searching for the best move
    pub async fn go(&mut self, depth: Option<u32>, movetime: Option<u64>) -> Result<()> {
        let cmd = match (depth, movetime) {
            (Some(d), _) => format!("go depth {}", d),
            (_, Some(t)) => format!("go movetime {}", t),
            _ => "go depth 15".to_string(),  // Default depth
        };
        self.send_command(&cmd).await
    }

    /// Stop the current search
    pub async fn stop(&mut self) -> Result<()> {
        self.send_command("stop").await
    }

    /// Receive and parse the next response from the engine
    pub async fn recv_response(&mut self) -> Option<EngineResponse> {
        let line = self.stdout_rx.recv().await?;
        Some(parse_engine_line(&line))
    }

    /// Try to receive a response without blocking
    pub fn try_recv_response(&mut self) -> Option<EngineResponse> {
        match self.stdout_rx.try_recv() {
            Ok(line) => Some(parse_engine_line(&line)),
            Err(_) => None,
        }
    }

    /// Quit the engine gracefully
    pub async fn quit(mut self) -> Result<()> {
        self.send_command("quit").await?;
        timeout(Duration::from_secs(3), self.child.wait())
            .await
            .context("Timeout waiting for engine to quit")??;
        Ok(())
    }
}

/// Parse a line from the engine into a response
fn parse_engine_line(line: &str) -> EngineResponse {
    if line.starts_with("bestmove") {
        parse_bestmove(line)
    } else if line == "readyok" {
        EngineResponse::Ready
    } else if line.starts_with("info") {
        parse_info(line)
    } else {
        EngineResponse::Error(line.to_string())
    }
}

/// Parse a bestmove line
fn parse_bestmove(line: &str) -> EngineResponse {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let mv= parts.get(1).unwrap_or(&"").to_string();
    let ponder = if parts.get(2) == Some(&"ponder") {
        parts.get(3).map(|s| s.to_string())
    } else {
        None
    };
    EngineResponse::BestMove { mv, ponder }
}

/// Parse an info line from the engine
fn parse_info(line: &str) -> EngineResponse {
    let mut depth = 0;
    let mut score = 0;
    let mut nodes = 0;
    let mut nps = 0;
    let mut pv = Vec::new();

    let parts: Vec<&str> = line.split_whitespace().collect();
    let mut i = 0;

    while i < parts.len() {
        match parts[i] {
            "depth" => {
                if let Some(val) = parts.get(i + 1) {
                    depth = val.parse().unwrap_or(0);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "score" => {
                if let Some(score_type) = parts.get(i + 1) {
                    if score_type == &"cp" {
                        if let Some(val) = parts.get(i + 2) {
                            score = val.parse().unwrap_or(0);
                            i += 3;
                        } else {
                            i += 2;
                        }
                    } else if score_type == &"mate" {
                        if let Some(val) = parts.get(i + 2) {
                            let mate_in: i32 = val.parse().unwrap_or(0);
                            // Convert mate score to centipawns for display
                            score = if mate_in == 0 { 10000 } else { -10000 };
                            i += 3;
                        } else {
                            i += 2;
                        }
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            "nodes" => {
                if let Some(val) = parts.get(i + 1) {
                    nodes = val.parse().unwrap_or(0);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "nps" => {
                if let Some(val) = parts.get(i + 1) {
                    nps = val.parse().unwrap_or(0);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "pv" => {
                // Collect all remaining moves as the principal variation
                pv = parts[i + 1..].iter().map(|s| s.to_string()).collect();
                break;
            }
            _ => i += 1,
        }
    }

    EngineResponse::Info {
        depth,
        score,
        nodes,
        nps,
        pv,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bestmove() {
        let line = "bestmove e2e4 ponder e7e5";
        match parse_bestmove(line) {
            EngineResponse::BestMove { mv, ponder } => {
                assert_eq!(mv, "e2e4");
                assert_eq!(ponder, Some("e7e5".to_string()));
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_parse_info() {
        let line = "info depth 15 score cp 34 nodes 1234567 nps 500000 pv e2e4 e7e5 g1f3";
        match parse_info(line) {
            EngineResponse::Info {
                depth,
                score,
                nodes,
                nps,
                pv,
            } => {
                assert_eq!(depth, 15);
                assert_eq!(score, 34);
                assert_eq!(nodes, 1234567);
                assert_eq!(nps, 500000);
                assert_eq!(pv, vec!["e2e4", "e7e5", "g1f3"]);
            }
            _ => panic!("Wrong response type"),
        }
    }
}