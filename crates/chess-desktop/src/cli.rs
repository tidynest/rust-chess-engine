use anyhow::Result;
use chess_core::{ChessEngine, GameState, display, notation};
use std::io::{self, Write};

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
                    }
                }
            }
        }
    }

    Ok(())
}
