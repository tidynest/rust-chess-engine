# Changelog

Newest first. Items refer to `docs/AUDIT_2026-09-09.md` where one exists.

## Unreleased

- `ChessApp::takes_moves` is the one check for whether a move may be made
  now; the board, the move box and "Play the best move" each had their own
  copy.

## 0.1.6 - 2026-10-05

- The CLI no longer asks "Play again? (y/n)" when a game ends, where any
  answer but `y` quit and the game was lost. It says the game is over and
  keeps taking commands, so `pgn` prints the finished game, `undo` takes
  moves back and `new` starts another. Moves are refused until then, which
  matters after a draw, where legal moves still exist.

- Ticking "Play vs Computer" during an analysis, with the computer to
  move, stops the analysis and lets the computer move at once. It used to
  wait for the analysis to finish, which at a high depth could take a
  long time.

- An `e.p.` after an en passant capture, as in `exd6 e.p.`, no longer
  stops a PGN from loading at that move. The move box accepts it too.

- The move box, the CLI and PGN import read SAN written with more or less
  than it needs, such as `ed5`, `Ng1f3`, `Ng1-f3` or `Pe4`, or an `x` on a
  move that takes nothing. A spelling is read only when exactly one legal
  move fits it, so `Nd2` with two knights able to go there is still
  refused.

- PGN files open when they start with a byte order mark, as files saved by
  Windows editors often do, and when they are not UTF-8, such as older
  exports with player names in Latin-1. Both used to fail: the first with
  a bad move, the second with "stream did not contain valid UTF-8".

## 0.1.5 - 2026-10-05

- A test plays random games from five start positions, including one with
  Black to move and two built for promotions and castling. Every legal
  move on the way must read back from its SAN and UCI, and every game from
  its PGN.

- In analysis, the arrows, the engine's lines and "Play the best move"
  show only lines searched in the position on screen. Right after a move
  or a jump through the history they could show the old position's best
  move for a moment, and play it if it was legal in the new one. With
  Stockfish stopped, the old line stayed for good. The score is kept
  across the change as before, so the eval bar does not flicker.

- After "Try again" restarts Stockfish, analysis asks the new engine for
  the number of lines again. It used to go on assuming the old engine's
  setting, so with three lines chosen the new one showed only one.

- A piece being dragged when the flag falls, or when a takeback or an
  engine move changes the position, is let go. After a flag fall it used
  to stay stuck where the pointer was, with its own square empty, into the
  next game.

- The settings file can no longer set a depth or time limit outside the
  sliders' ranges. Stockfish reads `go depth 0` and `go movetime 0` as no
  limit, so a hand-edited `0` left the computer thinking for good. A
  window size of `nan` or `inf` falls back to the default.

- Under ten seconds the clock cuts the tenths instead of rounding them. It
  used to read `0:10.0` between 9.95 and 10 seconds, and could show a
  tenth more than was left.

- Analysis runs again when its depth, time limit or mode changes, as it
  already did for the number of lines. A finished analysis used to keep
  its old depth until the position changed.

- A flag fall or a resignation closes the promotion picker and drops the
  selected piece. Picking a piece after the flag fell used to play the
  move anyway, after the game had ended. The resign prompt closes too,
  and a new game closes it, where before it stayed up and offered to
  resign the game just started.

## 0.1.4 - 2026-09-27

- Save game, Copy PGN and the CLI's `pgn` write the whole game, with its
  real result, whichever move is on screen. After stepping back they used
  to stop at the move shown, so a saved game could lose its last moves,
  and a resigned game kept its result on a shortened list. Save also works
  while the start position is on screen.

- The CLI exits when its input ends, by Ctrl+D or at the end of piped
  commands. It used to redraw the board in an endless loop.

- A game set up from a FEN, or opened from a PGN with one, numbers its
  moves from the FEN's move number in the move list and the Position
  panel. Both used to count from 1, and with Black to move the move list
  paired each Black move with the White move after it.

- Two games saved in the same second both keep their files. The second
  save used to overwrite the first; it is now named with a `_2` suffix.

- The eval bar is greyed out until a search reports a score. When the
  engine refused every search, the bar sat at an even split with no
  number, which looked like an equal position.

- In the CLI, an engine that answers with an illegal move no longer ends
  the program. The CLI says what it sent and carries on without an
  opponent, as it already did for an engine that stops.

- In the CLI, `undo` against Stockfish takes back its reply and your move
  together, and `redo` replays both. It used to take back only the reply,
  so your next move was made for Stockfish's side and it answered for
  yours.

- The strength options go to the engine only when the strength changes,
  not before every search. An engine without `UCI_LimitStrength`, such as
  Stockfish before version 11, now plays at full strength instead of
  refusing every move.

- After the engine refused an option, the next option it refused went
  unreported: the check for the second one read the first one's "ready"
  reply and passed. Each refusal is now reported with its own command.

- When the engine refuses a search, for instance because it has no
  `UCI_Elo` option, the notice bar says so and the board takes moves
  again. The board used to wait for a reply that never came, and in a game
  against the computer nothing could be played.

- A game clock started on an opened PGN now follows takebacks. It used to
  lose track of the moves: undo left the time as it was, and a look back
  at the start stopped the clock for the side to move until they moved.

- A line Stockfish prints during a search that the app does not expect,
  such as a refused command, now appears in the notice bar. It used to be
  dropped without a word. Normal searches print none of these.

- The eval bar only shows while Stockfish is running. When the engine
  failed during analysis, the bar used to stay up at an even split with no
  number, which looked like an equal position. It also stays hidden while
  the engine is still starting.

- `master` on GitHub is protected: pull requests need the CI jobs to pass,
  and nobody can force-push to it or delete it.

## 0.1.3 - 2026-09-27

- A test feeds 5,000 randomly damaged FEN strings, PGN files and moves to
  the parsers and checks that none of them panics. It uses a fixed seed,
  so a failure reproduces.

- In the CLI, a Stockfish that stops mid-game no longer ends the program
  and the game with it. The CLI prints why and carries on without an
  opponent; `play` starts a new engine.

- The repository tracks only RustRover's shared spelling dictionary and
  inspection profile from `.idea/`. The module file and the rest were one
  machine's settings and pointed at folders that no longer exist.

- The minimum Rust version is 1.95, not 1.88. egui 0.36 needs 1.95, so
  older compilers already failed, with an error about a dependency. A new
  CI job builds with the version `Cargo.toml` names.

- Linux builds need only `libasound2-dev` and `pkg-config`. The xcb,
  xkbcommon and OpenSSL headers listed in the README and installed by CI
  and the release workflow were left over from older dependencies; nothing
  in the build links them.

- When Stockfish stops, the engine panel says why when the engine said so.
  The app now reads its stderr, which used to be thrown away, and keeps the
  first error line from either stream. A Stockfish that cannot find its
  network file used to leave "engine closed its output"; it now shows the
  engine's own "Network evaluation parameters ... must be available".

- The app holds at most 256 unread lines from Stockfish. If it falls
  behind, Stockfish waits for it instead of the app's memory growing, and
  quitting no longer waits on an engine stuck writing to a full pipe.

- The release workflow can create a release on its own. It used to fail at
  that step because the publish job has no copy of the tag to read notes
  from; it now reads the tag message from GitHub and leaves out the subject
  line, which is already the title.

## 0.1.2 - 2026-09-27

- When the release workflow has to create the GitHub release itself, it
  titles it with the bare version, `0.1.2` rather than `v0.1.2`, like the
  earlier releases.

## 0.1.1 - 2026-09-26

- A release workflow builds `chess-gui` and `chess-cli` for Linux, macOS and
  Windows when a version tag is pushed and attaches them, with checksums, to
  the GitHub release. The Linux build comes from Ubuntu 22.04 so it runs on
  older distributions. On Windows the release GUI opens no console window.
- Softer sound cues. Each tone fades in over 4 ms so it no longer clicks,
  carries a quieter octave above it, and sits lower: moves on C5 instead of
  880 Hz, check on G5 instead of 1100 Hz, and a quieter low-time tick at
  1000 Hz instead of 1500 Hz.

## 0.1.0 - 2026-09-26

The first tagged release.

### Sixteenth round

- The `chess` crate is gone; move generation, legality and every board type
  now come from `cozy-chess` 0.3.4 (MIT, no `unsafe`, no dependencies). That
  closes the last three advisories, `failure` and the build-only `rand` 0.7,
  and drops their ignores. cozy-chess keeps the halfmove clock and the move
  number, so a FEN start numbers its moves correctly and the fifty-move rule
  no longer needs a workaround. Castling, which cozy-chess writes as the king
  taking its own rook, is turned into the king's landing square in one place,
  `chess_core::moves`, for UCI, SAN and the board.
- Sound: a click per move, a lower pair for a capture, a ping for check, a
  falling three-note end, and a tick per second under ten on the clock.
  Synthesised with rodio's playback feature alone; View > Sound switches it,
  and the choice is remembered.
- The CI workflow runs with read-only repository permissions, which clears
  the three CodeQL alerts that asked for it.

### Fifteenth round

- "Play the best move" under the analysis lines.
- Crate descriptions in the manifests; `docs/DECISIONS.md` lists the three
  open owner decisions with their trade-offs.

### Fourteenth round

- A box under the board takes a typed move, `e4`, `Nf3` or `e2e4`.
- A PGN file holding several games opens its first game instead of failing.
- README: Stockfish 16 or newer for the Elo limit.

### Thirteenth round

- CI installs Stockfish and runs the engine-link test against it.
- The window size is remembered between runs.
- CI badge in the README.

### Twelfth round

- `chess-gui game.pgn` opens the game at start.
- Position Info shows the move number, the side to play, the plies since the
  last capture or pawn move, and a selectable FEN.
- Eval bar label and curve tests; status notes on both audit documents.

### Eleventh round

- Computer plays: Both. Stockfish plays itself; browsing the history pauses
  it, End resumes it. Resign is off in that mode.
- The move list scrolls the current move into view when the position
  changes.
- Escape drops the selection and closes the resign prompt.

### Tenth round

- Resign asks first.
- Clock presets 1+0, 3+2, 5+3, 10+0 and 15+10.
- "Try again" starts the engine thread afresh when Stockfish could not be
  started.

### Ninth round

- Stockfish's "No such option" and "Unknown command" replies fail the
  command instead of vanishing behind its `readyok`, so a refused option
  reaches the status panel.
- Ctrl+S saves the game.
- A test runs a clock past zero and checks the flag fall ends the game.
- README screenshot shows analysis with three lines and the opening name.

### Eighth round

- chess-core's own `Color`, `PieceType`, `Piece`, `Square` and `Move`, the
  `GameState` trait and the `ChessEngine` wrapper are gone; everything speaks
  the `chess` crate's types. `notation::parse_uci` reads long algebraic moves
  by matching the legal moves and serves the CLI and the GUI alike;
  `GameHistory::uci_position` moved in from the GUI.
- The CLI keeps a `GameHistory`: `redo`, `pgn`, the draw rules, the opening
  name, and SAN that works. The old loop lowercased the input before parsing,
  so `Nf3` was rejected as `nf3`. Stockfish gets the full move list from the
  CLI too.
- Strength is set as an Elo (Stockfish's `UCI_LimitStrength` and `UCI_Elo`,
  1320 to 3190) behind a "Limit strength" box, instead of the skill level.
  The setting key is `engine_elo`; an old `skill_level` line is ignored.

### Seventh round

- The opening's name and ECO code, from the Lichess opening table (CC0,
  3810 rows in `crates/chess-core/data/openings.tsv`), in the left panel. A
  test plays every row through the PGN reader and finds it again.

### Sixth round

- Analysis mode shows up to five engine lines (MultiPV): Lines under Engine
  Settings, remembered between runs. Each line is listed with its score and
  drawn as an arrow, the best one boldest. Play always searches one line.
- Engine errors (a refused option, a search that could not start, an illegal
  move from the engine) appear in the status panel instead of on stderr.
- The last-move highlight follows the history, so it is right after undo,
  redo, a jump or a loaded game. It used to keep the undone move.
- `StockfishEngine::initialise` no longer sets Threads 4 and Hash 128; the
  GUI sends its own values, the CLI keeps Stockfish's defaults.
- A test maps every square back from its centre, flipped and not.

### Fifth round

- egui and eframe 0.36.2. The App trait hands the panels a root `Ui`, the
  default renderer is wgpu, and `paste` left the lockfile along with its
  advisory ignore.
- Save game writes the PGN to `$XDG_DATA_HOME/rust-chess-engine/games/`,
  named by the moment it was saved; Open game lists the twenty newest and
  plays one through. The PGN Date tag carries the real date. Both use UTC.
- Undo, redo and jumps put both clocks back to where they stood at that ply.
- Pieces slide to their square over 150 ms, for played moves and for the
  ply stepped by undo or redo; a piece dropped by drag stays put.

### Fourth round

- Game clocks: minutes plus a Fischer increment, off by default, set under
  Clock for the next game. A flag fall ends the game and sets the PGN result.
  With a clock the engine searches on `go wtime btime winc binc` and manages
  its own time; `SearchLimit` replaces the depth and movetime pair.
- Resign button; Ctrl+Z, Ctrl+Y and Ctrl+N shortcuts.
- Stockfish Threads and Hash are settings, sent as setoption and remembered.
  The thread default is half the machine's threads, at most four.
- CLI `play` answers your moves with Stockfish.
- README screenshot.

### Third round

- Analysis mode: "Analyse position" evaluates whatever is on screen, also
  while browsing the history, and draws the engine's move as an arrow. It
  never blocks the board and never plays its result.
- Settings persist: theme, engine limits, skill level, eval bar, captured
  layout and the computer's colour, as `key = value` lines under
  `$XDG_CONFIG_HOME/rust-chess-engine/settings`.
- Load PGN from the Game menu; comments, variations, glyphs and results are
  stripped. `notation::parse_san` reads SAN by matching the formatter over the
  legal moves, which also covers en passant, and a test proves every legal
  move has a SAN of its own.
- CLI accepts SAN (`Nf3`, `O-O`, `exd5`) and a `fen` command.
- King in check and the drag target square are tinted; ticking "Play vs
  Computer" or changing its colour puts the human at the bottom.
- Promotion picker and captured pieces use the vector piece set.
- Engine path: `CHESS_STOCKFISH`, then PATH, `/usr/games`, `/opt/homebrew/bin`,
  `/usr/local/bin`, `stockfish.exe`. The skill label no longer invents an Elo.
- The GUI reads the position only from `GameHistory` (the duplicate
  `ChessEngine` is gone), dependencies build optimised in dev, CI runs
  cargo-deny, the board's minimum size is 240 px.

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
