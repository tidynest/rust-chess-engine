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
- Undo, redo, and click any move in the history to jump to it.
- Play Stockfish as White or Black, with a depth or time limit and skill level 0 to 20.
- Evaluation bar, depth, node count and principal variation while the engine thinks.
- Captured pieces in Lichess or Chess.com style.
- CLI: long algebraic input (`e2e4`, `e7e8q`) and a legal-move list.

## Known limitations

- Promotion from the GUI always gives a queen. The CLI accepts `e7e8n`.
- No draw detection beyond stalemate. No clocks. No PGN.
- The engine receives the position as a FEN string only, so it cannot see repetitions.
- Engine path is `stockfish` on `PATH`; 4 threads and 128 MB hash are fixed.

The full list and the plan to fix them is in `docs/AUDIT_2026-09-09.md`.

## Layout

- `crates/chess-core`: domain types, `GameHistory` with undo and redo, SAN and long-algebraic notation.
- `crates/chess-engine`: async UCI client for the Stockfish process.
- `crates/chess-desktop`: the egui GUI (`chess-gui`) and the CLI (`chess-cli`).

## Licence

MIT. See `LICENSE`.
