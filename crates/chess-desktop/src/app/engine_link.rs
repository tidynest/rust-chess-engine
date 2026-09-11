//! The Stockfish thread and the messages that cross to and from it.
//!
//! The UI tags every search with an id and the thread echoes it on each
//! reply, so a reply to a position the user has since left is dropped rather
//! than played.

use chess_engine::{EngineResponse, SearchLimit, StockfishEngine};
use eframe::egui::Context;
use std::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedReceiver;

/// A search the UI wants run.
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// Echoed on every reply.
    pub id: u64,
    /// Everything after `position `, e.g. `startpos moves e2e4 e7e5`.
    pub position: String,
    pub limit: SearchLimit,
    pub skill_level: i32,
}

/// What the UI can ask the thread to do.
#[derive(Debug, Clone)]
pub enum EngineCommand {
    Search(SearchRequest),
    /// Abort the running search. Its `bestmove` still arrives under the old id.
    Stop,
    /// `ucinewgame`, so the engine clears its tables.
    NewGame,
    /// `setoption name <name> value <value>`, applied between searches.
    SetOption {
        name: String,
        value: String,
    },
    Quit,
}

/// What the thread reports back.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    Ready,
    Search {
        id: u64,
        response: EngineResponse,
    },
    /// The process could not be started or went away. Nothing follows.
    Failed(String),
}

/// Where the engine thread is in its life.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineStatus {
    Starting,
    Ready,
    Failed(String),
}

/// Start the engine thread. It runs until `Quit` arrives or the command
/// sender is dropped, and asks `ctx` to repaint after every event.
pub fn spawn(
    commands: UnboundedReceiver<EngineCommand>,
    events: Sender<EngineEvent>,
    ctx: Context,
) {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let emit = |event| {
            let _ = events.send(event);
            ctx.request_repaint();
        };
        match runtime {
            Ok(runtime) => runtime.block_on(run(commands, &emit)),
            Err(e) => emit(EngineEvent::Failed(format!("engine runtime: {e}"))),
        }
    });
}

async fn run(mut commands: UnboundedReceiver<EngineCommand>, emit: &dyn Fn(EngineEvent)) {
    let mut engine = match start().await {
        Ok(engine) => engine,
        Err(e) => return emit(EngineEvent::Failed(format!("{e:#}"))),
    };
    emit(EngineEvent::Ready);

    // Id of the search Stockfish is working on, if any.
    let mut current: Option<u64> = None;

    loop {
        tokio::select! {
            command = commands.recv() => match command {
                None | Some(EngineCommand::Quit) => break,
                Some(EngineCommand::Stop) => {
                    let _ = engine.stop().await;
                }
                Some(EngineCommand::SetOption { name, value }) => {
                    finish_search(&mut engine, &mut current, emit).await;
                    if let Err(e) = engine.set_option(&name, &value).await {
                        eprintln!("engine: {e:#}");
                    }
                }
                Some(EngineCommand::NewGame) => {
                    finish_search(&mut engine, &mut current, emit).await;
                    let _ = engine.new_game().await;
                }
                Some(EngineCommand::Search(request)) => {
                    finish_search(&mut engine, &mut current, emit).await;
                    if let Err(e) = start_search(&mut engine, &request).await {
                        eprintln!("engine: {e:#}");
                        continue;
                    }
                    current = Some(request.id);
                }
            },
            response = engine.recv_response() => match response {
                None => {
                    emit(EngineEvent::Failed("engine closed its output".to_owned()));
                    return;
                }
                Some(response) => forward(&mut current, response, emit),
            },
        }
    }

    let _ = engine.quit().await;
}

/// Start the engine named by `CHESS_STOCKFISH`, else the first of the
/// usual names and places that spawns.
async fn start() -> anyhow::Result<StockfishEngine> {
    let candidates = match std::env::var("CHESS_STOCKFISH") {
        Ok(path) => vec![path],
        Err(_) => [
            "stockfish",
            "/usr/games/stockfish",
            "/opt/homebrew/bin/stockfish",
            "/usr/local/bin/stockfish",
            "stockfish.exe",
        ]
        .map(str::to_owned)
        .to_vec(),
    };
    let mut last_error = None;
    for path in &candidates {
        match StockfishEngine::new(path).await {
            Ok(mut engine) => {
                engine.initialise().await?;
                return Ok(engine);
            }
            Err(e) => last_error = Some(e),
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("no engine path to try")))
}

async fn start_search(engine: &mut StockfishEngine, request: &SearchRequest) -> anyhow::Result<()> {
    engine
        .send_command(&format!(
            "setoption name Skill Level value {}",
            request.skill_level.clamp(0, 20)
        ))
        .await?;
    engine.wait_ready().await?;
    engine.set_position(&request.position).await?;
    engine.go(request.limit).await
}

/// Stop the running search, if any, and forward its replies up to and
/// including `bestmove`, so the next command sees a quiet engine.
async fn finish_search(
    engine: &mut StockfishEngine,
    current: &mut Option<u64>,
    emit: &dyn Fn(EngineEvent),
) {
    if current.is_none() {
        return;
    }
    let _ = engine.stop().await;
    while current.is_some() {
        match engine.recv_response().await {
            Some(response) => forward(current, response, emit),
            None => return,
        }
    }
}

/// Tag a reply with the running search's id; `bestmove` ends that search.
fn forward(current: &mut Option<u64>, response: EngineResponse, emit: &dyn Fn(EngineEvent)) {
    let Some(id) = *current else {
        return;
    };
    if matches!(response, EngineResponse::BestMove { .. }) {
        *current = None;
    }
    emit(EngineEvent::Search { id, response });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn search(id: u64, position: &str, depth: u32) -> EngineCommand {
        EngineCommand::Search(SearchRequest {
            id,
            position: position.to_owned(),
            limit: SearchLimit::Depth(depth),
            skill_level: 20,
        })
    }

    #[test]
    #[ignore = "needs stockfish on PATH"]
    fn a_new_search_stops_the_old_one_and_replies_in_order() {
        let (tx, commands) = tokio::sync::mpsc::unbounded_channel();
        let (events, rx) = std::sync::mpsc::channel();
        spawn(commands, events, Context::default());
        let recv = || rx.recv_timeout(Duration::from_secs(20)).unwrap();

        assert!(matches!(recv(), EngineEvent::Ready));

        // Depth 40 would take minutes; the second request must cut it short.
        tx.send(search(1, "startpos", 40)).unwrap();
        tx.send(search(2, "startpos moves e2e4", 4)).unwrap();

        let mut finished = Vec::new();
        while finished.len() < 2 {
            match recv() {
                EngineEvent::Search {
                    id,
                    response: EngineResponse::BestMove { mv, .. },
                } => {
                    assert!(mv.is_some());
                    finished.push(id);
                }
                EngineEvent::Search { id, .. } => {
                    assert_eq!(id, if finished.is_empty() { 1 } else { 2 });
                }
                other => panic!("{other:?}"),
            }
        }
        assert_eq!(finished, [1, 2]);

        tx.send(EngineCommand::Quit).unwrap();
    }
}
