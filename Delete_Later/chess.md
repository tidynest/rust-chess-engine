# Rust Chess Engine Development Guide: Quick Reference

## Critical Actions Required

1. **Three API updates** - `Rounding` → `CornerRadius`, remove `close_menu()`, add `StrokeKind`
2. **Undo/redo** - Store full board copies in `Vec<Board>`
3. **Clippy lints** - Enable strict configuration to catch 80% of bugs

---

## egui 0.32.3: API Migrations

### Breaking Changes

**Rounding → CornerRadius (v0.31.0)**
- `Rounding` type renamed to `CornerRadius`
- `f32` → `u8` for memory efficiency
- Replace: `Rounding::ZERO` → `CornerRadius::ZERO`
- Replace: `Rounding::same(4.0)` → `CornerRadius::same(4)`
- All `.rounding` fields → `.corner_radius`

**Menu Closure (v0.32.0)**
- `close_menu()` deprecated
- Menus auto-close on click by default
- Use `ui.close()` for programmatic closure

**StrokeKind Requirement (v0.31.0)**
- All `rect_stroke()` calls need positioning parameter
- Options: `StrokeKind::Inside`, `StrokeKind::Middle`, `StrokeKind::Outside`
- Use `StrokeKind::Inside` for board highlights

### Migration Examples

**File: `crates/chess-desktop/src/gui_app.rs`**

```rust
// OLD API
use egui::{Rounding, Color32};

fn draw_board_square(ui: &mut egui::Ui, rect: egui::Rect, highlight: bool) {
    let painter = ui.painter();
    let rounding = if highlight { Rounding::same(4.0) } else { Rounding::ZERO };
    
    painter.rect_filled(rect, rounding, Color32::LIGHT_GRAY);
    if highlight {
        painter.rect_stroke(rect, rounding, egui::Stroke::new(2.0, Color32::YELLOW));
    }
}

// NEW API
use egui::{CornerRadius, Color32, StrokeKind};

fn draw_board_square(ui: &mut egui::Ui, rect: egui::Rect, highlight: bool) {
    let painter = ui.painter();
    let corner_radius = if highlight { CornerRadius::same(4) } else { CornerRadius::ZERO };
    
    painter.rect_filled(rect, corner_radius, Color32::LIGHT_GRAY);
    if highlight {
        painter.rect_stroke(
            rect, 
            corner_radius, 
            egui::Stroke::new(2.0, Color32::YELLOW),
            StrokeKind::Inside  // NEW: Required parameter
        );
    }
}
```

**File: `crates/chess-desktop/src/gui_app.rs` (Menu updates)**

```rust
// OLD: Manual closing
ui.menu_button("Game", |ui| {
    if ui.button("New Game").clicked() {
        start_new_game();
        ui.close_menu();  // REMOVE THIS
    }
});

// NEW: Auto-close (default)
ui.menu_button("Game", |ui| {
    if ui.button("New Game").clicked() {
        start_new_game();
        // Auto-closes automatically
    }
});

// NEW: Custom behavior for multi-selection menus
use egui::{MenuButton, MenuConfig, PopupCloseBehavior};

MenuButton::new("Options")
    .config(MenuConfig::default()
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside))
    .ui(ui, |ui| {
        ui.checkbox(&mut settings.show_coordinates, "Show Coordinates");
        ui.checkbox(&mut settings.highlight_moves, "Highlight Legal Moves");
    });
```

### Optional Enhancements

- **Atoms system** - Mix images + text in buttons
- **Scene container** - Built-in pan/zoom for analysis mode

---

## Undo/Redo Implementation

### Why Copy-on-Make Works

- Chess crate 3.2.0 uses immutable `Board` struct (~64-100 bytes)
- **No `undo_move()` method** - designed around immutability
- Every move creates new `Board` via `board.make_move_new()`
- Store history in `Vec<Board>` at application level

### Implementation

**File: `crates/chess-core/src/game.rs` (create new file)**

```rust
use chess::{Board, ChessMove};

pub struct ChessGame {
    positions: Vec<Board>,
    moves: Vec<ChessMove>,
    current_index: usize,
}

impl ChessGame {
    pub fn new() -> Self {
        Self {
            positions: vec![Board::default()],
            moves: Vec::new(),
            current_index: 0,
        }
    }
    
    pub fn current(&self) -> &Board {
        &self.positions[self.current_index]
    }
    
    pub fn make_move(&mut self, mv: ChessMove) {
        // Truncate future history on new move
        self.positions.truncate(self.current_index + 1);
        self.moves.truncate(self.current_index);
        
        let new_board = self.current().make_move_new(mv);
        self.positions.push(new_board);
        self.moves.push(mv);
        self.current_index += 1;
    }
    
    pub fn undo(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            false
        }
    }
    
    pub fn redo(&mut self) -> bool {
        if self.current_index < self.positions.len() - 1 {
            self.current_index += 1;
            true
        } else {
            false
        }
    }
    
    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }
    
    pub fn can_redo(&self) -> bool {
        self.current_index < self.positions.len() - 1
    }
}
```

### Engine Search Pattern

**File: `crates/chess-engine/src/search.rs` (create new file)**

```rust
use chess::{Board, MoveGen};

fn negamax(board: &Board, depth: u8) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }
    
    let mut best_score = i32::MIN;
    let moves = MoveGen::new_legal(board);
    
    for mv in moves {
        let new_board = board.make_move_new(mv);  // Copy-on-make
        let score = -negamax(&new_board, depth - 1);
        best_score = best_score.max(score);
    }
    
    best_score
}
```

---

## Testing Strategy

### Unit Tests

**File: `crates/chess-core/src/engine.rs` (add tests module)**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chess::{Board, Square, ChessMove};
    
    #[test]
    fn test_initial_position_moves() {
        let board = Board::default();
        let moves = MoveGen::new_legal(&board).collect::<Vec<_>>();
        assert_eq!(moves.len(), 20);
    }
    
    #[test]
    fn test_en_passant_capture() {
        let board = Board::from_str(
            "rnbqkbnr/ppp2ppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1"
        ).unwrap();
        
        let ep_move = ChessMove::new(Square::E5, Square::D6, None);
        assert!(MoveGen::new_legal(&board).any(|m| m == ep_move));
    }
    
    #[test]
    fn test_checkmate_detection() {
        let board = Board::from_str(
            "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3"
        ).unwrap();
        
        assert_eq!(MoveGen::new_legal(&board).count(), 0);
    }
}
```

### Perft Tests

**File: `tests/perft_tests.rs` (create new file)**

```rust
use chess::{Board, MoveGen};

fn perft(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    
    let mut nodes = 0;
    let moves = MoveGen::new_legal(board);
    
    for mv in moves {
        let new_board = board.make_move_new(mv);
        nodes += perft(&new_board, depth - 1);
    }
    
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn perft_initial_position() {
        let board = Board::default();
        assert_eq!(perft(&board, 1), 20);
        assert_eq!(perft(&board, 2), 400);
        assert_eq!(perft(&board, 3), 8_902);
        assert_eq!(perft(&board, 4), 197_281);
    }
    
    #[test]
    #[ignore]  // Slow test
    fn perft_kiwipete() {
        let board = Board::from_str(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        ).unwrap();
        
        assert_eq!(perft(&board, 1), 48);
        assert_eq!(perft(&board, 2), 2_039);
        assert_eq!(perft(&board, 3), 97_862);
    }
}
```

### Property-Based Tests

**File: `Cargo.toml` (workspace root)**

```toml
[workspace.dependencies]
proptest = "1.5"
```

**File: `crates/chess-core/Cargo.toml`**

```toml
[dev-dependencies]
proptest = { workspace = true }
```

**File: `crates/chess-core/src/lib.rs` (add tests)**

```rust
#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn evaluation_consistent(moves in prop::collection::vec(any::<u16>(), 1..20)) {
            let mut board = Board::default();
            
            for _ in moves.iter() {
                let legal_moves: Vec<_> = MoveGen::new_legal(&board).collect();
                if legal_moves.is_empty() { break; }
                
                let mv = legal_moves[moves[0] as usize % legal_moves.len()];
                board = board.make_move_new(mv);
            }
            
            let eval1 = evaluate(&board);
            let eval2 = evaluate(&board);
            prop_assert_eq!(eval1, eval2);
        }
    }
}
```

### Benchmarks

**File: `Cargo.toml` (workspace root)**

```toml
[workspace.dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

**File: `benches/movegen_bench.rs` (create new file)**

```toml
# Add to chess-core/Cargo.toml
[dev-dependencies]
criterion = { workspace = true }

[[bench]]
name = "movegen_bench"
harness = false
```

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use chess::{Board, MoveGen};

fn bench_move_generation(c: &mut Criterion) {
    let positions = vec![
        ("Starting", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        ("Kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
        ("Endgame", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
    ];
    
    let mut group = c.benchmark_group("movegen");
    for (name, fen) in positions {
        let board = Board::from_str(fen).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(name), &board, |b, board| {
            b.iter(|| {
                let moves: Vec<_> = MoveGen::new_legal(black_box(board)).collect();
                moves.len()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_move_generation);
criterion_main!(benches);
```

### GUI Testing Pattern

**File: `crates/chess-desktop/src/logic.rs` (create new file)**

```rust
// Testable: Pure game logic
use chess::{Board, Square, ChessMove, MoveGen};

pub struct ChessLogic {
    pub game: ChessGame,
}

impl ChessLogic {
    pub fn handle_square_click(&mut self, square: Square) -> Result<(), GameError> {
        // All logic here - fully testable
        Ok(())
    }
    
    pub fn get_legal_moves_for(&self, square: Square) -> Vec<ChessMove> {
        MoveGen::new_legal(self.game.current())
            .filter(|m| m.get_source() == square)
            .collect()
    }
}

// Tests in same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_legal_moves_filter() {
        let logic = ChessLogic { game: ChessGame::new() };
        let moves = logic.get_legal_moves_for(Square::E2);
        assert_eq!(moves.len(), 2); // e2-e3 and e2-e4
    }
}
```

**File: `crates/chess-desktop/src/gui_app.rs` (presentation layer)**

```rust
// Not testable: Pure presentation
fn render_board(ui: &mut egui::Ui, logic: &ChessLogic) {
    for square in all_squares() {
        if ui.button(square.to_string()).clicked() {
            let _ = logic.handle_square_click(square);
        }
    }
}
```

---

## Code Quality Configuration

### Clippy Lints

**File: `Cargo.toml` (workspace root)**

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
dead_code = "warn"
unused_imports = "warn"
unused_variables = "warn"

[workspace.lints.clippy]
# Always fix
correctness = "deny"
suspicious = "deny"

# Improve
complexity = "warn"
perf = "warn"
style = "warn"

# Performance critical
cast_lossless = "warn"
inefficient_to_string = "warn"
needless_pass_by_value = "warn"
trivially_copy_pass_by_ref = "warn"

# Safety critical
indexing_slicing = "warn"
panic = "warn"
unwrap_used = "warn"
expect_used = "warn"

# Code quality
missing_const_for_fn = "warn"
unnecessary_wraps = "warn"
```

Run: `RUSTFLAGS="-D warnings" cargo clippy --all-targets --all-features`

### Release Optimization

**File: `Cargo.toml` (workspace root)**

```toml
[profile.release]
opt-level = 3              # Maximum optimization
lto = "fat"               # Link-time optimization (10-20% faster)
codegen-units = 1         # Better optimization
panic = "abort"           # Smaller binary
strip = true              # Remove debug symbols
```

Build: `RUSTFLAGS="-C target-cpu=native" cargo build --release`

### Performance Patterns

**File: `crates/chess-engine/src/lib.rs`**

```rust
use arrayvec::ArrayVec;

const MAX_MOVES: usize = 256;

// Avoid allocations
fn generate_moves(board: &Board, moves: &mut ArrayVec<ChessMove, MAX_MOVES>) {
    moves.clear();
    for mv in MoveGen::new_legal(board) {
        moves.push(mv);
    }
}

// Inline hot functions
#[inline(always)]
fn pop_lsb(bitboard: &mut u64) -> u8 {
    let square = bitboard.trailing_zeros() as u8;
    *bitboard &= *bitboard - 1;
    square
}
```

### Type Safety

**File: `crates/chess-core/src/types.rs` (create new file)**

```rust
// Newtype pattern for type safety
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    pub const fn new(rank: u8, file: u8) -> Self {
        Square(rank * 8 + file)
    }
    
    pub const A1: Square = Square::new(0, 0);
    pub const H8: Square = Square::new(7, 7);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Score(i32);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bitboard(u64);
```

---

## Action Checklist

### Immediate (Today)

- [ ] Update `Rounding` → `CornerRadius` in `gui_app.rs`
- [ ] Remove all `close_menu()` calls
- [ ] Add `StrokeKind::Inside` to `rect_stroke()` calls
- [ ] Run `cargo clippy --all-targets` and fix warnings

### This Week

- [ ] Create `ChessGame` struct with undo/redo in `chess-core/src/game.rs`
- [ ] Add Clippy lints to workspace `Cargo.toml`
- [ ] Write perft tests in `tests/perft_tests.rs`
- [ ] Separate GUI logic into `logic.rs` and `gui_app.rs`

### Before Phase 4

- [ ] Configure release profile optimization
- [ ] Set up criterion benchmarks
- [ ] Add property-based tests with proptest
- [ ] Create test organization structure (`tests/`, `benches/`, `fixtures/`)
- [ ] Run `cargo test -- --include-ignored` to verify all tests pass

### Ongoing

- [ ] Run `cargo clippy` before every commit
- [ ] Run `cargo test` during development
- [ ] Run `cargo bench` weekly
- [ ] Use `cargo audit` for dependency checks
- [ ] Profile with `cargo flamegraph` before optimizing

---

## Test Commands Reference

```bash
# Fast tests (development)
cargo test

# All tests including slow ones (pre-commit)
cargo test -- --include-ignored

# Specific test
cargo test test_checkmate_detection

# Benchmarks
cargo bench

# Clippy with strict warnings
RUSTFLAGS="-D warnings" cargo clippy --all-targets --all-features

# Release build with optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Profiling
cargo install flamegraph
cargo flamegraph --bin chess-gui

# Security audit
cargo audit
```

---

## File Structure

```
chess-engine/
├── Cargo.toml                    # Workspace + lints + profiles
├── crates/
│   ├── chess-core/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── engine.rs
│   │   │   ├── game.rs          # NEW: Undo/redo
│   │   │   ├── types.rs         # NEW: Type safety
│   │   │   └── ...
│   │   └── Cargo.toml
│   ├── chess-engine/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── search.rs        # NEW: Search algorithm
│   │   └── Cargo.toml
│   └── chess-desktop/
│       ├── src/
│       │   ├── cli.rs
│       │   ├── gui.rs
│       │   ├── gui_app.rs       # UPDATE: API changes
│       │   └── logic.rs         # NEW: Testable logic
│       └── Cargo.toml
├── tests/                        # NEW: Integration tests
│   └── perft_tests.rs
├── benches/                      # NEW: Benchmarks
│   └── movegen_bench.rs
└── fixtures/                     # NEW: Test data
    └── positions.fen
```