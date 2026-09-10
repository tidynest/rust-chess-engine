# rust-chess-engine

Desktop chess GUI and terminal CLI in Rust. Plays against a locally installed
[Stockfish](https://stockfishchess.org/) over UCI. Move generation and
validation come from the `chess` crate. The project contains no search engine
of its own.

## Prerequisites

- Rust 1.88 or newer (`rustup update stable`).
- Stockfish on `PATH` for the computer opponent. Arch: `pacman -S stockfish`.
  Debian and Ubuntu: `apt install stockfish`. macOS: `brew install stockfish`.
  Windows: download from stockfishchess.org and put `stockfish.exe` on `PATH`.
- Linux build dependencies for eframe (Debian names):
  `libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`.

## Build and run

```bash
cargo run --release --bin chess-gui
```

```bash
cargo run --bin chess-cli
```

```bash
cargo test --workspace
```

```bash
cargo run -p chess-engine --example test_stockfish
```

The last one only checks that Stockfish answers over UCI.

## Features

- Click or drag to move. Legal targets are marked, the last move is highlighted.
  A pawn reaching the last rank asks which piece it becomes.
- Undo, redo, and click any move in the history to jump to it. Against the
  computer, undo takes back your move and its reply together.
- Play Stockfish as White or Black, with a depth or time limit and skill level 0 to 20.
  The engine gets the full move list, so it sees repetitions and the 50-move rule.
- Evaluation bar, depth, node count and principal variation while the engine thinks.
- Captured pieces in Lichess or Chess.com style.
- CLI: long algebraic input (`e2e4`, `e7e8q`), a legal-move list and undo.

## Known limitations

- No draw detection beyond stalemate. No clocks. No PGN.
- The engine is `stockfish` on `PATH` unless `CHESS_STOCKFISH` names another
  binary; 4 threads and 128 MB hash are fixed.
- Pieces are drawn from the system font, so they look different on every machine.

The full list and the plan to fix them is in `docs/AUDIT_2026-09-09.md`;
what has been done since is in `CHANGELOG.md`.

## Layout

- `crates/chess-core`: domain types, `GameHistory` with undo, redo and SAN, long-algebraic notation.
- `crates/chess-engine`: async UCI client for the Stockfish process.
- `crates/chess-desktop`: the egui GUI (`chess-gui`) and the CLI (`chess-cli`).
  `app/engine_link.rs` is the engine thread; the UI talks to it with tagged requests.

The test that drives a real Stockfish is ignored by default:

```bash
cargo test -p chess-desktop -- --ignored
```

## Licence

MIT. See `LICENSE`.
