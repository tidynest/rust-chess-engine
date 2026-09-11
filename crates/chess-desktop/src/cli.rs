use anyhow::{Context, Result};
use chess_core::{ChessEngine, GameState, display, notation};
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

    /// The engine's move for `fen`, in long algebraic notation.
    // ponytail: a bare FEN, so the engine cannot see repetitions here.
    fn best_move(&mut self, fen: &str) -> Result<String> {
        let Self { runtime, engine } = self;
        runtime.block_on(async {
            engine.set_position(&format!("fen {fen}")).await?;
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
    println!("    fen       - Print the position as FEN");
    println!("    fen <fen> - Set up a position");
    println!("    play      - Let Stockfish answer your moves (again to stop)");
    println!();
}

fn show_legal_moves(engine: &ChessEngine) {
    let moves = engine.legal_moves();
    println!("\nLegal moves ({} total):", moves.len());

    let mut move_strings: Vec<String> = moves.iter().map(notation::to_algebraic).collect();
    move_strings.sort();

    // Display in columns
    for chunk in move_strings.chunks(10) {
        println!("  {}", chunk.join("  "));
    }
}

fn main() -> Result<()> {
    println!("♔ Welcome to Rust Chess Engine! ♚");
    println!("Type 'help' for commands\n");

    let mut engine = ChessEngine::new();
    // Positions before each move, newest last; popping one is an undo.
    let mut history: Vec<ChessEngine> = Vec::new();
    let mut opponent: Option<Opponent> = None;

    loop {
        println!("\n{}", display::display_board(&engine));
        println!("{}", display::display_status(&engine));

        if engine.is_checkmate() || engine.is_stalemate() {
            println!("\nGame Over!");
            print!("Play again? (y/n): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                engine = ChessEngine::new();
                history.clear();
                continue;
            } else {
                break;
            }
        }

        print!("\nEnter move: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input.to_lowercase().as_str() {
            "quit" | "exit" | "q" => {
                println!("Thanks for playing!");
                break;
            }
            "help" | "h" | "?" => {
                print_help();
            }
            "new" => {
                engine = ChessEngine::new();
                history.clear();
                println!("New game has started!");
            }
            "moves" | "m" => {
                show_legal_moves(&engine);
            }
            "undo" | "u" => match history.pop() {
                Some(previous) => engine = previous,
                None => println!("Nothing to undo"),
            },
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
            "fen" => println!("{}", engine.board()),
            command if command.starts_with("fen ") => match ChessEngine::from_fen(&input[4..]) {
                Ok(position) => {
                    engine = position;
                    history.clear();
                }
                Err(e) => println!("Invalid position: {e}"),
            },
            move_str => {
                if move_str.is_empty() {
                    continue;
                }

                let before = engine.clone();
                let played = match notation::parse_algebraic(move_str) {
                    Some(mv) => engine.make_move(mv),
                    None => engine.make_san(move_str),
                };
                match played {
                    Ok(()) => {
                        history.push(before);
                        println!("Move played: {}", move_str);
                    }
                    Err(e) => {
                        println!("Invalid move: {}", e);
                        println!("Type 'moves' to see legal moves, 'help' for the formats");
                        continue;
                    }
                }

                if let Some(stockfish) = &mut opponent
                    && !engine.is_checkmate()
                    && !engine.is_stalemate()
                {
                    let reply = stockfish.best_move(&engine.board().to_string())?;
                    let mv = notation::parse_algebraic(&reply)
                        .with_context(|| format!("engine sent {reply:?}"))?;
                    engine.make_move(mv)?;
                    println!("Stockfish played: {reply}");
                }
            }
        }
    }

    Ok(())
}
