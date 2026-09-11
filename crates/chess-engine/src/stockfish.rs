//! Stockfish UCI client. Spawns the process, forwards commands over stdin and
//! parses the lines it prints on stdout.

use anyhow::{Context, Result};
use std::process::Stdio;
use std::str::SplitWhitespace;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};

/// Search score from the point of view of the side to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    /// Centipawns; 100 is one pawn.
    Cp(i32),
    /// Moves until mate. Positive means the side to move delivers it, zero
    /// means the side to move is already mated.
    Mate(i32),
}

impl Score {
    /// The same score seen from the other side.
    pub fn flipped(self) -> Self {
        match self {
            Self::Cp(cp) => Self::Cp(-cp),
            // ponytail: Mate(0) cannot flip; it only occurs for positions no
            // caller searches.
            Self::Mate(moves) => Self::Mate(-moves),
        }
    }
}

/// A line from the engine that the caller cares about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineResponse {
    /// `readyok`.
    Ready,
    /// A search update carrying an exact score.
    Info {
        depth: u32,
        score: Score,
        nodes: u64,
        nps: u64,
        pv: Vec<String>,
    },
    /// Search finished. `mv` is `None` when the position has no legal move.
    BestMove {
        mv: Option<String>,
        ponder: Option<String>,
    },
    /// Anything else, verbatim.
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
            .kill_on_drop(true) // Ensure cleanup
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

    /// Set a UCI option and wait until the engine has taken it.
    pub async fn set_option(&mut self, name: &str, value: impl std::fmt::Display) -> Result<()> {
        self.send_command(&format!("setoption name {name} value {value}"))
            .await?;
        self.wait_ready().await
    }

    /// Start a new game
    pub async fn new_game(&mut self) -> Result<()> {
        self.send_command("ucinewgame").await?;
        self.wait_ready().await?;
        Ok(())
    }

    /// Set the board position
    pub async fn set_position(&mut self, position_cmd: &str) -> Result<()> {
        self.send_command(&format!("position {}", position_cmd))
            .await?;
        Ok(())
    }

    /// Start searching for the best move
    pub async fn go(&mut self, depth: Option<u32>, movetime: Option<u64>) -> Result<()> {
        let cmd = match (depth, movetime) {
            (Some(d), _) => format!("go depth {}", d),
            (_, Some(t)) => format!("go movetime {}", t),
            _ => "go depth 15".to_string(), // Default depth
        };
        self.send_command(&cmd).await
    }

    /// Stop the current search
    pub async fn stop(&mut self) -> Result<()> {
        self.send_command("stop").await
    }

    /// Next response the caller cares about; lines with nothing in them
    /// (`info string`, bound-only scores, `currmove` progress) are skipped.
    /// `None` once the engine has closed its stdout.
    pub async fn recv_response(&mut self) -> Option<EngineResponse> {
        loop {
            let line = self.stdout_rx.recv().await?;
            if let Some(response) = parse_engine_line(&line) {
                return Some(response);
            }
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

/// Parse one stdout line. `None` means the line carries nothing to report.
fn parse_engine_line(line: &str) -> Option<EngineResponse> {
    let mut words = line.split_whitespace();
    match words.next()? {
        "bestmove" => Some(parse_bestmove(words)),
        "readyok" => Some(EngineResponse::Ready),
        "info" => parse_info(words),
        _ => Some(EngineResponse::Error(line.to_string())),
    }
}

/// Words after `bestmove`: a move or `(none)`, optionally `ponder <move>`.
fn parse_bestmove(mut words: SplitWhitespace<'_>) -> EngineResponse {
    let mv = words.next().filter(|mv| *mv != "(none)").map(str::to_owned);
    let ponder = (words.next() == Some("ponder"))
        .then(|| words.next())
        .flatten()
        .map(str::to_owned);
    EngineResponse::BestMove { mv, ponder }
}

/// Words after `info`. Only lines with an exact score for the first PV are
/// reported; everything else would flash meaningless numbers in a UI.
fn parse_info(mut words: SplitWhitespace<'_>) -> Option<EngineResponse> {
    let mut depth = 0;
    let mut score = None;
    let mut nodes = 0;
    let mut nps = 0;
    let mut pv = Vec::new();

    while let Some(key) = words.next() {
        match key {
            "string" | "lowerbound" | "upperbound" => return None,
            "multipv" if words.next()? != "1" => return None,
            "depth" => depth = words.next()?.parse().ok()?,
            "nodes" => nodes = words.next()?.parse().ok()?,
            "nps" => nps = words.next()?.parse().ok()?,
            "score" => {
                let kind = words.next()?;
                let value = words.next()?.parse().ok()?;
                score = Some(match kind {
                    "cp" => Score::Cp(value),
                    "mate" => Score::Mate(value),
                    _ => return None,
                });
            }
            "pv" => {
                pv = words.map(str::to_owned).collect();
                break;
            }
            _ => {}
        }
    }

    Some(EngineResponse::Info {
        depth,
        score: score?,
        nodes,
        nps,
        pv,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(line: &str) -> (u32, Score, Vec<String>) {
        match parse_engine_line(line) {
            Some(EngineResponse::Info {
                depth, score, pv, ..
            }) => (depth, score, pv),
            other => panic!("{line:?} parsed to {other:?}"),
        }
    }

    #[test]
    fn test_parse_bestmove() {
        assert_eq!(
            parse_engine_line("bestmove e2e4 ponder e7e5"),
            Some(EngineResponse::BestMove {
                mv: Some("e2e4".into()),
                ponder: Some("e7e5".into()),
            })
        );
        assert_eq!(
            parse_engine_line("bestmove (none)"),
            Some(EngineResponse::BestMove {
                mv: None,
                ponder: None,
            })
        );
    }

    #[test]
    fn test_parse_info() {
        let line = "info depth 15 seldepth 22 multipv 1 score cp 34 nodes 1234567 nps 500000 pv e2e4 e7e5 g1f3";
        assert_eq!(
            parse_engine_line(line),
            Some(EngineResponse::Info {
                depth: 15,
                score: Score::Cp(34),
                nodes: 1234567,
                nps: 500000,
                pv: vec!["e2e4".into(), "e7e5".into(), "g1f3".into()],
            })
        );
    }

    #[test]
    fn test_parse_info_keeps_mate_sign() {
        assert_eq!(info("info depth 20 score mate 3").1, Score::Mate(3));
        assert_eq!(info("info depth 20 score mate -3").1, Score::Mate(-3));
        assert_eq!(Score::Mate(3).flipped(), Score::Mate(-3));
        assert_eq!(Score::Cp(-50).flipped(), Score::Cp(50));
    }

    #[test]
    fn test_parse_info_skips_lines_without_an_exact_score() {
        for line in [
            "info string NNUE evaluation using nn-1c0000000000.nnue",
            "info depth 12 currmove e2e4 currmovenumber 1",
            "info depth 18 score cp 40 lowerbound nodes 100 pv e2e4",
            "info depth 18 multipv 2 score cp 12 pv d2d4",
            "info depth 18 score cp notanumber pv e2e4",
        ] {
            assert_eq!(parse_engine_line(line), None, "{line:?}");
        }
    }

    #[test]
    fn test_parse_other_lines() {
        assert_eq!(parse_engine_line("readyok"), Some(EngineResponse::Ready));
        assert_eq!(
            parse_engine_line("id name Stockfish 18"),
            Some(EngineResponse::Error("id name Stockfish 18".into()))
        );
        assert_eq!(parse_engine_line(""), None);
    }
}
