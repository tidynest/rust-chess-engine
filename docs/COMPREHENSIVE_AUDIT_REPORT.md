# Comprehensive Codebase Audit Report

**Project:** Chess Engine (Rust Desktop Application)
**Date:** 2026-02-17
**Scope:** Full codebase audit across 3 crates, 30+ source files, build system, and documentation

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Chess Core Logic](#2-chess-core-logic)
3. [Engine / Stockfish Integration](#3-engine--stockfish-integration)
4. [Desktop GUI](#4-desktop-gui)
5. [Build System & Tests](#5-build-system--tests)
6. [Documentation](#6-documentation)
7. [Cross-Cutting Concerns](#7-cross-cutting-concerns)
8. [Prioritized Fix Plan](#8-prioritized-fix-plan)

---

## 1. Executive Summary

### Severity Totals (Deduplicated)

| Severity | Count | Description |
|----------|-------|-------------|
| **Critical** | 8 | Bugs causing incorrect behavior or missing core functionality |
| **Major** | 18 | Significant issues affecting correctness, usability, or maintainability |
| **Minor** | 30+ | Code quality, unused code, lint violations, minor bugs |
| **Suggestion** | 10+ | Enhancements, architecture improvements, future considerations |

### Top 5 Issues Requiring Immediate Attention

1. **No pawn promotion UI** -- Users cannot promote pawns via the GUI (board.rs)
2. **Mate score conversion is inverted** -- Engine displays wrong evaluation on mate positions (stockfish.rs:266)
3. **Theme system designed but entirely dead** -- 30+ hardcoded colors bypass the complete theme system (all UI files)
4. **`to_chess_move` ignores promotion field** -- Core engine may select wrong promotion piece (engine.rs:48-55)
5. **Engine thread cannot be cancelled** -- No way to stop an in-progress search; blocks shutdown (state.rs, stockfish.rs)

---

## 2. Chess Core Logic

**Crate:** `crates/chess-core/`
**Architecture:** Thin wrapper around the [`chess` crate v3.2.0](https://crates.io/crates/chess). Defines own domain types and converts to/from the underlying library.

### Critical Issues

| ID | File:Line | Issue | Fix |
|----|-----------|-------|-----|
| CORE-1 | `engine.rs:48-55` | **`to_chess_move` ignores promotion field.** When a pawn promotes, there are 4 legal moves with the same from/to squares. The method matches only on source/destination, returning whichever promotion `MoveGen` yields first. A player selecting "promote to knight" might silently get a queen. | Add promotion match to the filter: `m.get_promotion() == mv.promotion.map(convert)` |

### Major Issues

| ID | File:Line | Issue | Fix |
|----|-----------|-------|-----|
| CORE-2 | `traits.rs` | **No draw condition detection.** Missing 50-move rule, threefold repetition, and insufficient material detection. Games can continue indefinitely past draw conditions. | Add `is_draw_by_fifty_moves()`, `is_draw_by_repetition()`, `is_insufficient_material()` to `GameState` trait |
| CORE-3 | `game.rs:33-42` | **`GameHistory::make_move` accepts moves without legality validation.** `Board::make_move_new()` does not check legality -- passing an illegal move silently corrupts board state. | Validate via `MoveGen::new_legal` or document the precondition clearly |

### Minor Issues

| ID | File:Line | Issue | Fix |
|----|-----------|-------|-----|
| CORE-4 | `notation.rs:12-16` | Fragile `to_digit(18)` parsing trick -- panics in debug mode on malformed input (e.g., digit chars as file) | Use `bytes[0].checked_sub(b'a').filter(\|&f\| f < 8)?` |
| CORE-5 | `engine.rs:44` | `.unwrap()` on `Square::new()` violates workspace lint `unwrap_used = "warn"` | Use `.expect()` with explanation or safe conversion |
| CORE-6 | `notation.rs:45-51` | Silent discard of invalid promotion type (Pawn/King as promotion) | Return `None` or assert on invalid promotion types |
| CORE-7 | `notation.rs:68-69` | Casting `File` enum to `i8` relies on external crate's repr | Use `.to_index() as i8` instead |
| CORE-8 | `Cargo.toml:10` | `anyhow` dependency declared but never imported in any source file | Remove unused dependency |
| CORE-9 | `engine.rs:26` | Doc comment missing closing parenthesis: "Get the underlying board (for GUI access" | Add `)` |
| CORE-10 | `engine.rs:39` | Typo: "Convert chess crate Square to out Square" | Change "out" to "our" |
| CORE-11 | `game.rs:384` | Test uses illegal move `Nb8-c7` (pawn on c7), silently corrupts board. Test passes because it only checks counts. | Fix test to use legal moves |
| CORE-12 | `engine.rs` | No `to_fen()` method to export current position | Add `pub fn to_fen(&self) -> String` |

### Suggestions

| ID | Issue | Notes |
|----|-------|-------|
| CORE-S1 | Dual type systems create mapping overhead and bug surface | Consider re-exporting `chess` crate types or fully committing to custom types |
| CORE-S2 | Perft tests validate the `chess` crate directly, not the `chess-core` wrapper | Add perft tests through `ChessEngine::legal_moves()` / `make_move()` interface |
| CORE-S3 | Missing `is_game_over()` and `fen()` on `GameState` trait | Add convenience methods |

---

## 3. Engine / Stockfish Integration

**Crates:** `crates/chess-engine/` and `crates/chess-desktop/src/app/engine_comm.rs`

### Critical Issues

| ID | File:Line | Issue | Fix |
|----|-----------|-------|-----|
| ENG-1 | `stockfish.rs:266` | **Mate score conversion is inverted.** `score = if mate_in == 0 { 10000 } else { -10000 }` discards the sign of `mate_in`. Both `mate 3` and `mate -3` become `-10000`. | `score = if mate_in > 0 { 10000 - mate_in } else { -10000 - mate_in }` |

### Major Issues

| ID | File:Line | Issue | Fix |
|----|-----------|-------|-----|
| ENG-2 | `state.rs:82-96` | **Silent engine failure, no user feedback.** If Stockfish isn't installed or crashes, `eprintln!` goes to stderr. GUI shows no indication -- "Play vs Computer" appears to do nothing. | Send error response through channel; show "Engine not available" in UI |
| ENG-3 | `stockfish.rs` | **No `Drop` implementation for graceful shutdown.** Engine process gets SIGKILL instead of UCI `quit` command when `StockfishEngine` is dropped. | Implement `Drop` that sends `quit` command |
| ENG-4 | `state.rs:120`, `stockfish.rs` | **No way to cancel in-progress search.** Once `go` is sent, the engine thread blocks until `bestmove` arrives. No `stop` command, no `tokio::select!`. Application cannot cleanly shut down during search. | Add `EngineCommand::Stop`; use `tokio::select!` to listen for commands while awaiting responses |
| ENG-5 | `state.rs:120-128` | **Engine thread blocks on search, ignores all commands including Quit.** The `while let Ok(cmd) = engine_rx.recv()` loop processes one command at a time. | Use `tokio::select!` to simultaneously await engine responses and new commands |
| ENG-6 | `stockfish.rs:173` | **FullStrength mode actually runs depth 15.** When `EngineMode::FullStrength` is selected, both `depth` and `movetime` are `None`. The `go()` method defaults to `"go depth 15"`. UI says "Maximum strength, no limits." | Change default to `"go infinite"` or `"go depth 40"` with stop mechanism |
| ENG-7 | `state.rs:181-192` | **`new_game()` does not send `ucinewgame` to Stockfish.** The UCI protocol requires `ucinewgame` between games so the engine clears hash tables. Without this, stale evaluations from the previous game may persist. | Add `EngineCommand::NewGame` and send `ucinewgame` |
| ENG-8 | `stockfish.rs:223` | **Empty `bestmove` string not handled.** If engine sends malformed `bestmove` line, empty string is passed as move, which silently fails to parse. | Return `EngineResponse::Error` if move string is missing/empty |
| ENG-9 | `stockfish.rs:120-121` | **Hash (128MB) and Threads (4) are hardcoded** in `initialise()`, not configurable from outside. May be inappropriate for systems with limited resources. | Accept configuration struct in `new()` or `initialise()` |
| ENG-10 | `state.rs:85` | **Stockfish path hardcoded as `"stockfish"`.** Relies on PATH lookup. No UI to specify custom path. Windows users may need `stockfish.exe`. | Add settings/preference for engine path with file browser |
| ENG-11 | `state.rs:83` | **`tokio::runtime::Runtime::new().unwrap()`** panics if runtime creation fails (e.g., out of file descriptors). Panic absorbed by thread. | Handle error, send failure notification through channel |

### Minor Issues

| ID | File:Line | Issue | Fix |
|----|-----------|-------|-----|
| ENG-12 | `stockfish.rs:208-217` | Non-error lines (e.g., `id`, `option`, `Stockfish` banner) classified as `EngineResponse::Error` | Add `Unknown(String)` variant or filter known harmless prefixes |
| ENG-13 | `stockfish.rs:70-71` | Unbounded channels can accumulate memory if consumer stalls. Engine produces thousands of `info` lines/sec. | Use bounded channels with backpressure |
| ENG-14 | `state.rs:83` | Full `tokio::runtime::Runtime` with `features = ["full"]` is overkill for I/O-only tasks | Use `Builder::new_current_thread().enable_all().build()` |
| ENG-15 | `state.rs:102-108` | Errors from `send_command` and `wait_ready` silently discarded with `let _ =` | At minimum log errors; ideally send error to UI |
| ENG-16 | `state.rs:57` | `engine_best_move: Option<String>` field declared but never written anywhere | Remove dead field |
| ENG-17 | `engine_comm.rs:227-239` | `sync_engine_from_history` has O(n^2) complexity -- rebuilds board from scratch for each move | Replay board once linearly, computing SAN at each step |
| ENG-18 | `stockfish.rs:61` | `Stdio::null()` for stderr -- engine error messages are lost | Capture stderr and log it |
| ENG-19 | `engine_comm.rs:102-106, 168-172` | Duplicated `Color` conversion logic in `request_engine_move` and `auto_request_engine_move` | Extract `fn current_chess_color(&self) -> ChessColor` |
| ENG-20 | `chess-engine/Cargo.toml:11-12` | `stockfish = "0.2.11"` and `vampirc-uci` declared but never imported in source code | Remove unused dependencies |
| ENG-21 | `chess-engine/lib.rs:12-25` | `Effect` and `GameResult` enums defined but never used | Remove or mark `#[allow(dead_code)]` |
| ENG-22 | `state.rs:68` | `_show_engine_settings` prefixed with underscore (dead code indicator) | Remove if unused, or wire up and remove underscore |

---

## 4. Desktop GUI

**Crate:** `crates/chess-desktop/`

### Critical Issues

| ID | File:Line | Issue | Fix |
|----|-----------|-------|-----|
| GUI-1 | `board.rs:289-316` | **No pawn promotion UI.** When a pawn reaches the 8th rank, `try_make_move` constructs a move string without promotion suffix. `parse_algebraic` cannot find a legal move. Promotion is silently rejected. No promotion dialog exists. | Add promotion detection (pawn on rank 7/2 moving to rank 8/1). Show promotion picker popup. Append chosen piece to move string. |
| GUI-2 | `state.rs:38-42, 149-153` | **Old color fields coexist with new theme system.** `ChessApp` has both `light_square_color`, `dark_square_color`, etc. (old) AND `theme`/`theme_variant` (new). Board renders with old fields. Theme is entirely dead code. | Remove old color fields (lines 38-42, 149-153). Replace all references with `self.theme.*` |
| GUI-3 | `theme.rs` (entire file) | **Theme system complete but 100% unused.** `theme.rs` has 3 well-designed theme variants with color, typography, and spacing tokens. No file in the codebase reads from the theme. | Integrate theme as described in CLAUDE.md |
| GUI-4 | `top_bar.rs:50-67` | **No theme selector menu.** View menu has old color pickers for `light_square_color`/`dark_square_color` instead of the theme variant selector described in CLAUDE.md. | Replace color pickers with `ThemeVariant::all()` radio buttons |

### Major Issues

| ID | File:Line | Issue | Fix |
|----|-----------|-------|-----|
| GUI-5 | `board.rs:289-316` | **Engine/history desync in `try_make_move`.** Calls `self.engine.make_move(mv)` then separately updates `game_history`. Engine advances one move ahead. `sync_engine_from_history` not called. | Call `sync_engine_from_history()` after move, or make `game_history` the single source of truth |
| GUI-6 | `board.rs:297-299` | **Dangerous fallback: `.unwrap_or(ChessMove::new(from, to, None))`** silently loses promotion data if legal move search fails. | Remove fallback; handle error explicitly |
| GUI-7 | `board.rs:82-93` | **Last-move highlighting painted in wrong order.** `last_move_color` overlay is painted UNDER the base square fill, so the highlight gets covered. | Paint base square first, then overlay `last_move_color` on top |
| GUI-8 | `state.rs:25-73` | **God Object: `ChessApp` has 35+ pub fields.** Mixes game logic, UI state, engine channels, and theme data. No encapsulation. | Group into sub-structs: `GameState`, `UiState`, `EngineState` |
| GUI-9 | Multiple files | **No user-visible error reporting.** All errors use `eprintln!` which goes to stderr, invisible to GUI users. | Add `error_message: Option<String>` field; display as toast/banner in UI |
| GUI-10 | `gui.rs:40-45` | **`configure_visuals` hardcodes dark theme.** Will conflict with Classic Monochrome theme (which is light). | Make `configure_visuals` theme-aware |

### Minor Issues (Hardcoded Colors Inventory)

**30+ hardcoded `Color32` values that bypass the theme system:**

| File | Count | Examples |
|------|-------|---------|
| `eval_bar.rs` | 8 | `Color32::from_rgb(40,40,40)`, `Color32::from_rgb(200,200,200)`, etc. |
| `board.rs` | 6 | Piece colors `(255,255,255)`, `(20,20,20)`, shadow, outline |
| `game_status.rs` | 3 | Check `(255,150,50)`, checkmate `(255,100,100)`, stalemate `(255,200,100)` |
| `right_panel.rs` | 5 | `Color32::from_rgb(100,200,255)`, `Color32::from_gray(120/160)` |
| `gui.rs` | 3 | Background `(30,30,35)`, surface `(25,25,30)`, text `gray(200)` |
| `material.rs` | 2 | `(200,200,200)`, `(100,100,100)` |
| `state.rs` | 5 | Old color field initializations |

### Other Minor Issues

| ID | File:Line | Issue |
|----|-----------|-------|
| GUI-11 | `board.rs:200-201` | `.unwrap()` on `interact_pointer_pos()` -- panics if pointer pos unavailable |
| GUI-12 | `board.rs:231` | `interact_pointer_pos().unwrap_or(Pos2::ZERO)` -- silently targets square a1 on failure |
| GUI-13 | `board.rs:393-409` | 8 extra text draws per piece for contrast outline = 256 extra text draws/frame |
| GUI-14 | `gui_app.rs` | Misleading filename (contains only tests, not app code). `cli.rs` imports it. |
| GUI-15 | `gui_app.rs:80-125` | Test helper `create_test_app()` lists all 35+ fields manually -- breaks on any field change |
| GUI-16 | Multiple | Spacing/font size hardcoded instead of using theme tokens (`8.0`, `16.0`, etc.) |
| GUI-17 | `material.rs:64` | Return value naming is confusing (captured pieces maps in reversed order) |
| GUI-18 | `state.rs:71-72` | No `set_theme()` method exists despite CLAUDE.md design specifying one |

### Suggestions

| ID | Issue |
|----|-------|
| GUI-S1 | No keyboard shortcuts (arrow keys for history navigation, Ctrl+Z undo) |
| GUI-S2 | No `ctx.request_repaint_after()` optimization -- redraws at 60fps even when idle |
| GUI-S3 | Material calculation iterates all 64 squares every frame (could cache) |
| GUI-S4 | Theme missing piece colors, shadow, outline, and `last_move_highlight` tokens |

---

## 5. Build System & Tests

### Critical Issues

| ID | Location | Issue | Fix |
|----|----------|-------|-----|
| BUILD-1 | Project root | **No CI/CD configuration.** No `.github/workflows/`, no `rust-toolchain.toml`, no CI of any kind. | Add GitHub Actions with: `cargo check`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo audit` |

### Major Issues

| ID | Location | Issue | Fix |
|----|----------|-------|-----|
| BUILD-2 | `chess-core/Cargo.toml` | **Workspace lints not inherited.** `[workspace.lints]` (forbid unsafe, clippy rules) does not apply to `chess-core`. | Add `[lints] workspace = true` |
| BUILD-3 | `chess-engine/Cargo.toml` | **Workspace lints not inherited** (same as above). | Add `[lints] workspace = true` |
| BUILD-4 | `chess-desktop/Cargo.toml` | **Workspace lints not inherited** (same as above). | Add `[lints] workspace = true` |
| BUILD-5 | `chess-core/` | **No unit tests for `ChessEngine`.** Functions `make_move`, `legal_moves`, `is_checkmate`, `is_stalemate`, `is_check`, `piece_at` are untested. | Add unit tests for the `GameState` implementation |

### Minor Issues

| ID | Location | Issue | Fix |
|----|----------|-------|-----|
| BUILD-6 | `Cargo.toml:25` | `tokio` with `features = ["full"]` compiles all features (net, fs, signal) when only io/process/sync/time are needed. Adds ~10-15s compile time. | Narrow to required features |
| BUILD-7 | `Cargo.toml:26` | `uciengine = "0.1.33"` in workspace dependencies but unused by any crate | Remove |
| BUILD-8 | `chess-engine/Cargo.toml` | `vampirc-uci` declared but never imported -- UCI parsing is manual in `stockfish.rs` | Remove (or use it to replace manual parsing) |
| BUILD-9 | `chess-core/Cargo.toml:11` | `anyhow` declared directly instead of `anyhow = { workspace = true }` | Use workspace reference |
| BUILD-10 | `Cargo.lock` | Two versions of `thiserror` compiled (1.0.69 and 2.0.17) -- caused by transitive deps | Awareness only; no action needed |

### Test Coverage

| Test Area | Count | Status |
|-----------|-------|--------|
| Perft tests (6 standard positions) | 14 (1 ignored) | Good coverage |
| `GameHistory` undo/redo/branching | 22 | Good |
| SAN notation formatting/parsing | 6 | Adequate |
| UCI response parsing | 2 | Minimal |
| GUI UCI move parsing, PV formatting | 5 | Adequate |
| **Total** | **49** | |

### Critical Test Gaps

| Area | Gap |
|------|-----|
| `ChessEngine` (`GameState` impl) | No tests at all |
| Coordinate transforms (`coords.rs`) | Board flip logic untested |
| Type conversions (`conversions.rs`) | Untested |
| Material counting (`material.rs`) | Untested |
| Display rendering (`display.rs`) | Untested |
| CLI game loop (`cli.rs`) | Untested |
| Engine integration (async) | Only parsing tested, not communication |

---

## 6. Documentation

### Critical Issues

| ID | Issue | Fix |
|----|-------|-----|
| DOC-1 | **No README.md** -- Project has zero public-facing documentation | Create README with project description, prerequisites (Rust, Stockfish on PATH), build/run instructions, features, and limitations |
| DOC-2 | **No LICENSE file** -- Legally ambiguous for anyone encountering the repo | Add appropriate license file |

### Major Issues

| ID | Issue | Fix |
|----|-------|-----|
| DOC-3 | **No CHANGELOG.md** -- 6 commits with substantial features, no formal change tracking | Create CHANGELOG.md |
| DOC-4 | **No architecture documentation** -- 3-crate workspace structure undocumented | Add architecture section to README or separate ARCHITECTURE.md |
| DOC-5 | **Stockfish dependency undocumented** -- External binary required but nowhere mentioned | Document in README prerequisites |
| DOC-6 | **CLAUDE.md describes unimplemented code as patterns** -- `set_theme()` method and theme menu described but don't exist. Could mislead developers. | Clarify what exists vs. what needs to be implemented |

### Minor Issues

| ID | Location | Issue |
|----|----------|-------|
| DOC-7 | `CLAUDE.md:29` | Typo: `F- Warm Minimal` -- stray `F` character before bullet |
| DOC-8 | `CLAUDE.md:4` | Stale date: "January 15, 2025" |
| DOC-9 | `Cargo.toml:24` | Stale comment: "Async for future Stockfish integration" -- integration already exists |
| DOC-10 | `chess-core/lib.rs:1` | False claim: "Pure chess domain logic with zero external dependencies" -- depends on `chess`, `thiserror`, `anyhow` |
| DOC-11 | `game.rs` | Public methods (`new`, `make_move`, `undo`, `redo`, etc.) missing `///` doc comments |
| DOC-12 | `.gitignore:3-4` | Lists `CLAUDE.md` but file is tracked in git (inconsistency) |

### Git History

- **6 commits**, clear development progression
- **No AI attributions found** in commit messages or bodies
- Commits align with documented project state

---

## 7. Cross-Cutting Concerns

### 7.1 Security

| ID | Issue | Status |
|----|-------|--------|
| SEC-1 | `unsafe_code = "forbid"` in workspace lints | Good (but not inherited -- see BUILD-2/3/4) |
| SEC-2 | Stockfish process spawned with user-controlled path | Minor risk -- path comes from hardcoded string, not user input |
| SEC-3 | No input sanitization on UCI move strings | Moves validated against legal move list -- safe |

### 7.2 Performance Hotspots

| ID | Location | Impact | Fix |
|----|----------|--------|-----|
| PERF-1 | `engine_comm.rs:227-239` | O(n^2) history rebuild on every sync | Replay once linearly |
| PERF-2 | `right_panel.rs:261-268` | Same O(n^2) pattern in `get_move_san` fallback | Use pre-computed cache |
| PERF-3 | `board.rs:393-409` | 320 text draws/frame for piece rendering (8 outline passes x ~32 pieces + shadows) | Single shadow + piece approach |
| PERF-4 | `material.rs:37-62` | Iterates all 64 squares every frame | Cache and invalidate on position change |
| PERF-5 | App repaints at 60fps even when idle | Wasted CPU | Use `request_repaint_after()` when idle |

### 7.3 Consistency Issues

| Issue | Files Affected |
|-------|---------------|
| Dual color systems (old fields + theme) | `state.rs`, `board.rs`, `top_bar.rs` |
| Mixed `std::sync::mpsc` and `tokio::sync::mpsc` | `state.rs`, `stockfish.rs` |
| Inconsistent error handling (some `eprintln!`, some `let _ =`, some `unwrap()`) | All crates |
| `chess_core::Color` to `chess::Color` conversion duplicated | `engine_comm.rs` |

---

## 8. Prioritized Fix Plan

### Phase 1: Critical Bugs (Correctness)

**Must fix before any other work. These cause incorrect behavior.**

| Priority | ID | Issue | Effort |
|----------|----|-------|--------|
| 1 | CORE-1 | Fix `to_chess_move` promotion matching | 1-line fix |
| 2 | ENG-1 | Fix mate score conversion (inverted logic) | 1-line fix |
| 3 | GUI-1 | Add pawn promotion UI dialog | ~50 lines |
| 4 | GUI-7 | Fix last-move highlight paint order | Reorder ~5 lines |
| 5 | GUI-5/6 | Fix engine/history desync in `try_make_move` | ~15 lines |

### Phase 2: Engine Reliability

**Engine integration has multiple issues that affect usability.**

| Priority | ID | Issue | Effort |
|----------|----|-------|--------|
| 6 | ENG-4/5 | Add search cancellation with `tokio::select!` | ~40 lines |
| 7 | ENG-6 | Fix FullStrength mode (depth 15 -> unlimited) | 1-line fix |
| 8 | ENG-2/GUI-9 | Add user-visible error reporting for engine failures | ~30 lines |
| 9 | ENG-7 | Send `ucinewgame` on new game | ~10 lines |
| 10 | ENG-3 | Implement `Drop` for graceful engine shutdown | ~15 lines |

### Phase 3: Theme System Integration

**The designed theme system needs to be wired in. Follows CLAUDE.md plan.**

| Priority | ID | Issue | Effort |
|----------|----|-------|--------|
| 11 | GUI-2 | Remove old color fields from `state.rs` | ~10 lines removed |
| 12 | GUI-18 | Add `set_theme()` method to `ChessApp` | ~5 lines |
| 13 | GUI-4 | Replace View menu color pickers with theme selector | ~15 lines |
| 14 | GUI-3 | Replace all 30+ hardcoded colors with theme tokens | ~60 edits across 7 files |
| 15 | GUI-10 | Make `configure_visuals` theme-aware | ~10 lines |

### Phase 4: Build & Infrastructure

| Priority | ID | Issue | Effort |
|----------|----|-------|--------|
| 16 | BUILD-1 | Set up CI/CD (GitHub Actions) | New workflow file |
| 17 | BUILD-2/3/4 | Inherit workspace lints in all crates | 3 x 1-line additions |
| 18 | BUILD-6/7/8 | Clean up unused dependencies, narrow tokio features | Cargo.toml edits |
| 19 | DOC-1/2 | Create README.md and LICENSE | New files |

### Phase 5: Core Logic & Tests

| Priority | ID | Issue | Effort |
|----------|----|-------|--------|
| 20 | CORE-2 | Add draw condition detection | ~80 lines |
| 21 | CORE-3 | Add legality validation to `GameHistory::make_move` | ~10 lines |
| 22 | BUILD-5 | Add unit tests for `ChessEngine` | ~100 lines |
| 23 | CORE-S2 | Add perft tests through wrapper API | ~50 lines |

### Phase 6: Polish & Performance

| Priority | ID | Issue | Effort |
|----------|----|-------|--------|
| 24 | PERF-1/2 | Fix O(n^2) history rebuild | ~30 lines |
| 25 | ENG-9/10 | Make engine settings configurable (hash, threads, path) | ~50 lines + UI |
| 26 | GUI-8 | Refactor `ChessApp` God Object into sub-structs | Significant refactor |
| 27 | GUI-S1 | Add keyboard shortcuts | ~30 lines |
| 28 | GUI-S2 | Optimize repaint frequency | ~5 lines |
| 29 | DOC-3/4 | Create CHANGELOG.md and architecture docs | New files |
| 30 | DOC-7-12 | Fix all documentation typos and staleness | Minor edits |

---

*Report generated by comprehensive codebase analysis across 5 parallel audit agents examining chess-core logic, engine/search, desktop GUI, build system, and documentation.*
