//! What the program writes it must read back: SAN and UCI for every legal
//! move, and whole games through PGN. Games are random from a fixed seed,
//! so a failure reproduces.

use chess_core::{GameHistory, moves, notation};
use cozy_chess::Board;

const STARTS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    // Castling both ways, pins, en passant and promotions within a few moves.
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    // Black to move, with an odd fullmove number.
    "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 17",
    // Pawns one step from promoting on both sides, knights for ambiguity.
    "1n2k1n1/P1P5/8/8/8/8/p1p5/1N2K1N1 w - - 0 1",
];

/// xorshift64, as in `garbage_input.rs`.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

/// A random game of up to `plies` moves, checking every legal move's SAN
/// and UCI on the way.
fn random_game(rng: &mut Rng, start: Board, plies: usize) -> GameHistory {
    let mut game = GameHistory::from_board(start);
    for _ in 0..plies {
        let board = game.current_board().clone();
        let legal = moves::legal_moves(&board);
        if legal.is_empty() || game.is_over() {
            break;
        }
        for &mv in &legal {
            let san = notation::format_move_san(&mv, &board);
            assert_eq!(
                notation::parse_san(&board, &san),
                Some(mv),
                "{san} in {board}"
            );
            let uci = moves::to_uci(&board, mv);
            assert_eq!(
                notation::parse_uci(&board, &uci),
                Some(mv),
                "{uci} in {board}"
            );
        }
        game.make_move(legal[rng.below(legal.len())]);
    }
    game
}

#[test]
fn moves_and_games_read_back_as_written() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for round in 0..30 {
        let start: Board = STARTS[round % STARTS.len()].parse().unwrap();
        let plies = 1 + rng.below(120);
        let game = random_game(&mut rng, start.clone(), plies);
        let pgn = game.pgn(Default::default());
        let read = GameHistory::from_pgn(&pgn).unwrap_or_else(|e| panic!("{e}\n{pgn}"));
        assert_eq!(read.current_moves(), game.current_moves(), "{pgn}");
        assert_eq!(read.start_board(), &start, "{pgn}");
        assert_eq!(read.pgn(Default::default()), pgn);
        assert_eq!(read.uci_position(), game.uci_position());
    }
}
