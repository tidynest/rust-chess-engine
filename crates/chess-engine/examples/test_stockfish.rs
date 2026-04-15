// Simple test to verify Stockfish integration works
// Run with: cargo run --package chess-engine --example test_stockfish

use chess_engine::{EngineResponse, StockfishEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Stockfish integration...\n");

    // Create engine instance
    println!("1. Starting Stockfish...");
    let mut engine = StockfishEngine::new("stockfish").await?;
    println!("   ✓ Engine process started\n");

    // Initialize
    println!("2. Initializing UCI mode...");
    engine.initialise().await?;
    println!("   ✓ Engine initialized\n");

    // Start new game
    println!("3. Starting new game...");
    engine.new_game().await?;
    println!("   ✓ New game started\n");

    // Set initial position
    println!("4. Setting initial position...");
    engine.set_position("startpos").await?;
    println!("   ✓ Position set\n");

    // Search for best move
    println!("5. Searching for best move (depth 15)...");
    engine.go(Some(15), None).await?;

    // Collect responses
    let mut best_move = None;
    let mut last_info = None;

    while let Some(response) = engine.recv_response().await {
        match response {
            EngineResponse::Info {
                depth,
                score,
                nodes,
                nps,
                pv,
            } => {
                // Only print every few depths to avoid spam
                if depth % 3 == 0 {
                    println!(
                        "   Depth {}: score {} cp, {} nodes, {} nps",
                        depth, score, nodes, nps
                    );
                    if !pv.is_empty() {
                        println!("   PV: {}", pv.join(" "));
                    }
                }
                last_info = Some((depth, score, nodes, pv));
            }
            EngineResponse::BestMove { mv, ponder } => {
                println!("\n   ✓ Best move found: {}", mv);
                if let Some(p) = ponder {
                    println!("   Suggested ponder move: {}", p);
                }
                best_move = Some(mv);
                break;
            }
            EngineResponse::Error(msg) => {
                println!("   Info: {}", msg);
            }
            _ => {}
        }
    }

    // Print summary
    println!("\n📊 Summary:");
    if let Some((depth, score, nodes, pv)) = last_info {
        println!("   Final depth: {}", depth);
        println!(
            "   Evaluation: {} centipawns ({:.2} pawns)",
            score,
            score as f32 / 100.0
        );
        println!("   Nodes searched: {}", nodes);
        if !pv.is_empty() {
            println!(
                "   Principal variation: {}",
                pv[..pv.len().min(5)].join(" ")
            );
        }
    }
    if let Some(mv) = best_move {
        println!("   Best move: {}", mv);
    }

    // Quit
    println!("\n6. Shutting down engine...");
    engine.quit().await?;
    println!("   ✓ Engine shut down cleanly\n");

    println!("✅ All tests passed!");

    Ok(())
}
