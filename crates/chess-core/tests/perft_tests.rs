// Perft (Performance Test) - Validates move generation correctness
// by counting all leaf nodes at a given depth
//
// Standard test positions from: https://www.chessprogramming.org/Perft_Results

use std::str::FromStr;
use chess::{Board, MoveGen};

/// Core perft function - recursively counts legal positions
fn perft(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let movegen = MoveGen::new_legal(board);

    for chess_move in movegen {
        let new_board = board.make_move_new(chess_move);
        nodes += perft(&new_board, depth - 1);
    }

    nodes
}

/// Perft with move breakdown (useful for debugging)
#[allow(dead_code)]
fn perft_divide(board: &Board, depth: u8) -> Vec<(String, u64)> {
    let mut results = Vec::new();
    let movegen = MoveGen::new_legal(board);

    for chess_move in movegen {
        let new_board = board.make_move_new(chess_move);
        let nodes = if depth > 1 {
            perft(&new_board, depth - 1)
        } else {
            1
        };
        results.push((format!("{}", chess_move), nodes));
    }

    results
}

#[test]
fn perft_initial_position_depth_1() {
    let board = Board::default();
    assert_eq!(perft(&board, 1), 20, "Initial position should have 20 moves");
}

#[test]
fn perft_initial_position_depth_2() {
    let board = Board::default();
    assert_eq!(
        perft(&board, 2),
        400,
        "Initial position at depth 2 should have 400 nodes"
    );
}

#[test]
fn perft_initial_position_depth_3() {
    let board = Board::default();
    assert_eq!(
        perft(&board, 3),
        8_902,
        "Initial position at depth 3 should have 8,902 nodes"
    );
}

#[test]
fn perft_initial_position_depth_4() {
    let board = Board::default();
    assert_eq!(
        perft(&board, 4),
        197_281,
        "Initial position at depth 4 should have 197,281 nodes"
    );
}

#[test]
#[ignore]  // Run with: cargo test -- --ignored
fn perft_initial_position_depth_5() {
    let board = Board::default();
    assert_eq!(
        perft(&board, 5),
        4_865_609,
        "Initial position at depth 5 should have 4,865,609 nodes"
    );
}

#[test]
fn perft_kiwipete_depth_1() {
    let board = Board::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
        .expect("Valid FEN");
    assert_eq!(perft(&board, 1), 48, "Kiwipete depth 1");
}

#[test]
fn perft_kiwipete_depth_2() {
    let board = Board::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
        .expect("Valid FEN");
    assert_eq!(perft(&board, 2), 2_039, "Kiwipete depth 2");
}

#[test]
fn perft_kiwipete_depth_3() {
    let board = Board::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
        .expect("Valid FEN");
    assert_eq!(perft(&board, 3), 97_862, "Kiwipete depth 3");
}

#[test]
fn perft_kiwipete_depth_4() {
    let board = Board::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
        .expect("Valid FEN");
    assert_eq!(perft(&board, 4), 4_085_603, "Kiwipete depth 4");
}

#[test]
fn perft_endgame_position() {
    let board = Board::from_str("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1")
        .expect("Valid FEN");
    assert_eq!(perft(&board, 1), 14, "Endgame position depth 1");
    assert_eq!(perft(&board, 2), 191, "Endgame position depth 2");
    assert_eq!(perft(&board, 3), 2_812, "Endgame position depth 3");
}

#[test]
fn perft_position_4() {
    let board = Board::from_str("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
        .expect("Valid FEN");
    assert_eq!(perft(&board, 1), 6, "Position 4 depth 1");
    assert_eq!(perft(&board, 2), 264, "Position 4 depth 2");
    assert_eq!(perft(&board, 3), 9_467, "Position 4 depth 3");
}

#[test]
fn perft_position_5() {
    let board =
        Board::from_str("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
            .expect("Valid FEN");
    assert_eq!(perft(&board, 1), 44, "Position 5 depth 1");
    assert_eq!(perft(&board, 2), 1_486, "Position 5 depth 2");
    assert_eq!(perft(&board, 3), 62_379, "Position 5 depth 3");
}

#[test]
fn perft_position_6() {
    let board = Board::from_str(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    )
        .expect("Valid FEN");
    assert_eq!(perft(&board, 1), 46, "Position 6 depth 1");
    assert_eq!(perft(&board, 2), 2_079, "Position 6 depth 2");
    assert_eq!(perft(&board, 3), 89_890, "Position 6 depth 3");
}

#[test]
fn debug_perft_divide_initial_position() {
    let board = Board::default();
    let results = perft_divide(&board, 2);

    println!("\nPerft divide for initial position at depth 2:");
    println!("{:<10} {}", "Move", "Nodes");
    println!("{}", "-".repeat(20));

    let mut total = 0;
    for (move_str, nodes) in &results {
        println!("{:<10} {}", move_str, nodes);
        total += nodes;
    }
    println!("{}", "-".repeat(20));
    println!("{:<10} {}", "Total", total);
}