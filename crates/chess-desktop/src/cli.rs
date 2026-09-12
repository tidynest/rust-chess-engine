use anyhow::{Context, Result};
use chess_core::{GameHistory, PgnTags, display, notation, openings};
use chess_engine::{EngineResponse, SearchLimit, StockfishEngine};
use std::io::{self, Write};

/// Stockfish driven synchronously from the prompt loop.
struct Opponent {
    runtime: tokio::runtime::Runtime,
    engine: StockfishEngine,
}

impl Opponent {
    fn start() -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let engine = runtime.block_on(async {
            let path = std::env::var("CHESS_STOCKFISH").unwrap_or_else(|_| "stockfish".to_owned());
            let mut engine = StockfishEngine::new(&path).await?;
            engine.initialise().await?;
            anyhow::Ok(engine)
        })?;
        Ok(Self { runtime, engine })
    }

    /// The engine's move after `position`, in long algebraic notation.
    fn best_move(&mut self, position: &str) -> Result<String> {
        let Self { runtime, engine } = self;
        runtime.block_on(async {
            engine.set_position(position).await?;
            engine.go(SearchLimit::Depth(12)).await?;
            while let Some(response) = engine.recv_response().await {
                if let EngineResponse::BestMove { mv, .. } = response {
                    return mv.context("the engine has no legal move");
                }
            }
            anyhow::bail!("the engine closed its output")
        })
    }
}

fn print_help() {
    println!("\n=== Chess Engine Commands ===");
    println!("  Move format: e2e4 or SAN such as Nf3, exd5, O-O, e8=Q");
    println!("  Promotion in long form: e7e8q (q=queen, r=rook, b=bishop, n=knight)");
    println!("  Commands:");
    println!("    help      - Show this help");
    println!("    quit      - Exit the game");
    println!("    new       - Start a new game");
    println!("    moves     - Show all legal moves");
    println!("    undo      - Take back the last move");
    println!("    redo      - Replay a move taken back");
    println!("    pgn       - Print the game as PGN");
    println!("    fen       - Print the position as FEN");
    println!("    fen <fen> - Set up a position");
    println!("    play      - Let Stockfish answer your moves (again to stop)");
    println!();
}

fn show_legal_moves(board: &chess::Board) {
    let mut moves: Vec<String> = chess::MoveGen::new_legal(board)
        .map(|mv| mv.to_string())
        .collect();
    moves.sort();
    println!("\nLegal moves ({} total):", moves.len());
    for chunk in moves.chunks(10) {
        println!("  {}", chunk.join("  "));
    }
}

/// Say what the last move of the history was.
fn announce(game: &GameHistory, who: &str) {
    let san = game
        .move_count()
        .checked_sub(1)
        .and_then(|index| game.san(index))
        .unwrap_or("?");
    println!("{who}: {san}");
}

fn main() -> Result<()> {
    println!("\u{2654} Welcome to Rust Chess Engine! \u{265a}");
    println!("Type 'help' for commands\n");

    let mut game = GameHistory::new();
    let mut opponent: Option<Opponent> = None;

    loop {
        println!("\n{}", display::board(game.current_board()));
        println!("{}", display::status(&game));
        if let Some(opening) = openings::of(&game) {
            println!("{} ({})", opening.name, opening.eco);
        }

        if game.is_over() {
            println!("\nGame Over!");
            print!("Play again? (y/n): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().eq_ignore_ascii_case("y") {
                game = GameHistory::new();
                continue;
            }
            break;
        }

        print!("\nEnter move: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input.to_lowercase().as_str() {
            "" => {}
            "quit" | "exit" | "q" => {
                println!("Thanks for playing!");
                break;
            }
            "help" | "h" | "?" => print_help(),
            "new" => {
                game = GameHistory::new();
                println!("New game has started!");
            }
            "moves" | "m" => show_legal_moves(game.current_board()),
            "undo" | "u" => {
                if !game.undo() {
                    println!("Nothing to undo");
                }
            }
            "redo" | "r" => {
                if !game.redo() {
                    println!("Nothing to redo");
                }
            }
            "pgn" => println!("{}", game.pgn(PgnTags::default())),
            "play" => match opponent.take() {
                Some(_) => println!("Stockfish stopped"),
                None => match Opponent::start() {
                    Ok(started) => {
                        opponent = Some(started);
                        println!("Stockfish will answer your moves");
                    }
                    Err(e) => println!("Could not start Stockfish: {e:#}"),
                },
            },
            "fen" => println!("{}", game.current_board()),
            command if command.starts_with("fen ") => {
                match GameHistory::from_fen(input[4..].trim()) {
                    Ok(position) => game = position,
                    Err(e) => println!("Invalid position: {e}"),
                }
            }
            _ => {
                // SAN is case-sensitive, so the move is read as typed.
                let board = game.current_board();
                let Some(mv) = notation::parse_move(board, input) else {
                    println!("Invalid move: {input}");
                    println!("Type 'moves' to see legal moves, 'help' for the formats");
                    continue;
                };
                game.make_move(mv);
                announce(&game, "Move played");

                if let Some(stockfish) = &mut opponent
                    && !game.is_over()
                {
                    let reply = stockfish.best_move(&game.uci_position())?;
                    let mv = notation::parse_uci(game.current_board(), &reply)
                        .with_context(|| format!("engine sent {reply:?}"))?;
                    game.make_move(mv);
                    announce(&game, "Stockfish played");
                }
            }
        }
    }

    Ok(())
}
