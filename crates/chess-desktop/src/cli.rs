mod gui_app;

use anyhow::Result;
use chess_core::{ChessEngine, GameState, display, notation};
use std::io::{self, Write};

fn print_help() {
    println!("\n=== Chess Engine Commands ===");
    println!("  Move format: e2e4, e7e5, etc.");
    println!("  Promotion: e7e8q (q=queen, r=rook, b=bishop, n=knight)");
    println!("  Commands:");
    println!("    help  - Show this help");
    println!("    quit  - Exit the game");
    println!("    new   - Start a new game");
    println!("    moves - Show all legal moves");
    println!("    undo  - Undo last move (not implemented yet)");
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
    let mut move_history = Vec::new();

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
                move_history.clear();
                continue;
            } else {
                break;
            }
        }

        print!("\nEnter move: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "quit" | "exit" | "q" => {
                println!("Thanks for playing!");
                break;
            }
            "help" | "h" | "?" => {
                print_help();
            }
            "new" => {
                engine = ChessEngine::new();
                move_history.clear();
                println!("New game has started!");
            }
            "moves" | "m" => {
                show_legal_moves(&engine);
            }
            "undo" | "u" => {
                println!("Undo not implemented yet");
            }
            move_str => {
                if move_str.is_empty() {
                    continue;
                }

                match notation::parse_algebraic(move_str) {
                    Some(mv) => match engine.make_move(mv) {
                        Ok(_) => {
                            move_history.push(move_str.to_string());
                            println!("Move played: {}", move_str);
                        }
                        Err(e) => {
                            println!("Invalid move: {}", e);
                            println!("Type 'moves' to see legal moves");
                        }
                    },
                    None => {
                        println!("Invalid move format. Use format like: e2e4");
                        println!("Type 'help' for more information");
                    }
                }
            }
        }
    }

    Ok(())
}
