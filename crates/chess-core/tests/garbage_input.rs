//! Text from files, the clipboard and the keyboard must be rejected or read,
//! never panic. Valid inputs are mutated at random from a fixed seed, which
//! stands in for a fuzzer and runs on stable in CI.

use chess_core::{GameHistory, notation};
use cozy_chess::Board;

const SEEDS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2P5/8/8/8/8/5k2/K7 w - e3 99 250",
    "[Event \"x\"]\n[FEN \"4k3/P7/8/8/8/8/8/4K3 w - - 0 1\"]\n\n1. a8=Q+ Kd7 *",
    "1. e4 {Najdorf} e5 (1... c5 2. Nf3) 2. Nf3 Nc6 3. Bb5 a6 $1 4. O-O ; x\n1-0",
    "e2e4 e7e8q O-O-O exd6 e.p. Nbd7 R1a3 12...Qxh2#",
];

/// Characters that trip careless parsers: separators, digits past the
/// board, multibyte text and a NUL.
const NOISE: &[char] = &[
    ' ', '/', '-', '.', '[', ']', '{', '}', '(', ')', '"', ';', '\n', '0', '8', '9', 'x', '=', '+',
    'K', 'q', 'p', 'e', 'é', '♔', '\0',
];

/// xorshift64: reproducible and needs no dependency.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

/// A seed with a few characters replaced, inserted or deleted.
fn mutate(rng: &mut Rng) -> String {
    let mut chars: Vec<char> = SEEDS[rng.below(SEEDS.len())].chars().collect();
    for _ in 0..=rng.below(4) {
        let at = rng.below(chars.len() + 1);
        let noise = NOISE[rng.below(NOISE.len())];
        match rng.below(3) {
            0 if at < chars.len() => chars[at] = noise,
            1 if at < chars.len() => {
                chars.remove(at);
            }
            _ => chars.insert(at, noise),
        }
    }
    chars.into_iter().collect()
}

#[test]
fn parsers_survive_mutated_input() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let board = Board::default();
    for _ in 0..5_000 {
        let text = mutate(&mut rng);
        let _ = GameHistory::from_fen(&text);
        let _ = notation::parse_move(&board, &text);
        for word in text.split_whitespace() {
            let _ = notation::parse_move(&board, word);
        }
        if let Ok(game) = GameHistory::from_pgn(&text) {
            let _ = game.pgn(Default::default());
        }
    }
}
