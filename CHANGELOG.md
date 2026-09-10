# Changelog

Newest first. Items refer to `docs/AUDIT_2026-09-09.md` where one exists.

## Unreleased

### Added since phase 1

- Draw detection: stalemate, threefold repetition, the fifty-move rule and
  insufficient material end the game and are named in the status line (CORE-2).
- Three themes from `theme.rs` wired through every panel, switchable under
  View > Theme; egui's own widgets follow the theme (GUI-2, GUI-3, GUI-4,
  GUI-10, GUI-18, phase 3).
- Keyboard: Left and Right arrows undo and redo, Home and End jump, F flips.
- Game menu: Copy FEN, Copy PGN (seven-tag roster, SetUp/FEN for custom
  starts, result), and Set up position from a FEN.
- `CHESS_STOCKFISH` names the engine binary (ENG-10). Closing the window
  quits the engine.
- The eval bar is a split bar on the Lichess curve, oriented by the board flip.
- Pieces are drawn from shapes in `ui/pieces.rs`, one triangulated silhouette
  per piece, instead of font glyphs, so they look the same everywhere.

### Changed since phase 1

- The GUI reads the position from `GameHistory` only; the duplicate
  `ChessEngine` on `ChessApp` and the conversions module are gone (phase 2).
- Jumping to the live end of the history lets the engine reply again.

### Fixed

- Mate scores keep their sign; the eval bar shows `M3` or `-M3` (3.1, ENG-1).
- Engine lines without an exact score (`info string`, `currmove`, bound results,
  extra PV lines) no longer flash depth 0 and 0.00 (3.1).
- Castling that gives check is written `O-O+` and `O-O#` (3.1).
- Promotions keep the piece asked for. A bare `e7e8` is illegal; the GUI asks
  for the piece (3.1, CORE-1).
- Undo and redo against the computer land on the human's turn, and the board
  no longer freezes on the computer's turn after browsing the history (3.1).
- Replies from a search started on an earlier position are dropped; the search
  is stopped when the position changes (3.1, ENG-4, ENG-5).
- Stockfish receives `position startpos moves ...` and `ucinewgame`, so it can
  see repetitions and the 50-move rule (3.1, ENG-7).
- The evaluation sign is derived from the position the search was started on (3.1).
- `parse_algebraic` rejects digits, off-board squares and unknown promotion
  letters instead of underflowing (3.1, CORE-4).
- Captured pieces are counted from the moves played; a promotion no longer
  shows as a captured pawn (3.1).
- The last-move highlight is painted over the square, not under it (GUI-7).
- The eval bar follows the board flip and keeps its label off the fill (3.2).
- A missing Stockfish is shown in the panel instead of a checkbox that does
  nothing (ENG-2, GUI-9).
- `bestmove (none)` is no longer treated as a move string (3.2).
- The engine thread starts on a current-thread runtime with an error path
  instead of `Runtime::new().unwrap()` (ENG-11).

### Added

- `README.md`, `LICENSE` (MIT), CI workflow, Dependabot, `.cargo/audit.toml`,
  `deny.toml`, `clippy.toml`, `rust-toolchain.toml`.
- `GameHistory` stores each move's SAN and exposes the moves with their boards.
- `ChessApp::headless` for tests; an ignored test drives a real Stockfish.
- CLI `undo`.
- `From` conversions between `chess_core::Square` and `chess::Square`.

### Changed

- Workspace lints apply to every crate; clippy is clean with `-D warnings`.
- Eight unused dependencies, three unimplemented traits and the
  `EngineCommand` type in `chess-engine` removed. The engine protocol lives in
  `chess-desktop/src/app/engine_link.rs`.
- `cargo update` cleared all five RUSTSEC vulnerabilities. The remaining
  warnings come from the unmaintained `chess` crate and are listed with
  reasons in `.cargo/audit.toml` and `deny.toml`.
