//! Game state management with undo/redo support

use chess::{Board, ChessMove};

/// Game state with full move history for undo/redo
pub struct GameHistory {
    positions: Vec<Board>,
    moves: Vec<ChessMove>,
    current_index: usize,
}

impl GameHistory {
    pub fn new() -> Self {
        Self {
            positions: vec![Board::default()],
            moves: Vec::new(),
            current_index: 0,
        }
    }

    pub fn from_board(board: Board) -> Self {
        Self {
            positions: vec![board],
            moves: Vec::new(),
            current_index: 0,
        }
    }

    pub fn current_board(&self) -> &Board {
        &self.positions[self.current_index]
    }

    pub fn make_move(&mut self, mv: ChessMove) {
        // Truncate future history when making a new move
        self.positions.truncate(self.current_index + 1);
        self.moves.truncate(self.current_index);

        let new_board = self.current_board().make_move_new(mv);
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

    pub fn move_count(&self) -> usize {
        self.current_index
    }

    pub fn get_move(&self, index: usize) -> Option<&ChessMove> {
        if index < self.current_index {
            self.moves.get(index)
        } else {
            None
        }
    }

    pub fn current_moves(&self) -> &[ChessMove] {
        &self.moves[..self.current_index]
    }
}

impl Default for GameHistory {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for GameHistory struct

#[cfg(test)]
mod tests {
    use super::*;
    use chess::{ChessMove, Square};
    use std::str::FromStr;

    // Helper function to create a simple move
    fn create_move(from: Square, to: Square) -> ChessMove {
        ChessMove::new(from, to, None)
    }

    // =============================================================================
    // Basic Functionality Tests
    // =============================================================================

    #[test]
    fn test_new_game_history() {
        let history = GameHistory::new();

        assert_eq!(history.move_count(), 0, "New game should have 0 moves");
        assert!(!history.can_undo(), "Cannot undo with no moves");
        assert!(!history.can_redo(), "Cannot redo with no moves");
        assert_eq!(
            history.current_board(),
            &Board::default(),
            "Should start at initial position"
        );
    }

    #[test]
    fn test_make_single_move() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);

        history.make_move(e2e4);

        assert_eq!(history.move_count(), 1, "Should have 1 move");
        assert!(history.can_undo(), "Should be able to undo");
        assert!(!history.can_redo(), "Should not be able to redo");
    }

    #[test]
    fn test_undo_single_move() {
        let mut history = GameHistory::new();
        let initial_board = history.current_board().clone();
        let e2e4 = create_move(Square::E2, Square::E4);

        history.make_move(e2e4);
        let result = history.undo();

        assert!(result, "Undo should succeed");
        assert_eq!(history.move_count(), 0, "Move count should be 0");
        assert!(!history.can_undo(), "Cannot undo further");
        assert!(history.can_redo(), "Should be able to redo");
        assert_eq!(
            history.current_board(),
            &initial_board,
            "Should return to initial position"
        );
    }

    #[test]
    fn test_redo_single_move() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);

        history.make_move(e2e4);
        let board_after_move = history.current_board().clone();
        history.undo();
        let result = history.redo();

        assert!(result, "Redo should succeed");
        assert_eq!(history.move_count(), 1, "Move count should be 1");
        assert!(history.can_undo(), "Should be able to undo");
        assert!(!history.can_redo(), "Cannot redo further");
        assert_eq!(
            history.current_board(),
            &board_after_move,
            "Should return to position after move"
        );
    }

    #[test]
    fn test_undo_at_start_fails() {
        let mut history = GameHistory::new();

        let result = history.undo();

        assert!(!result, "Undo should fail at start");
        assert_eq!(history.move_count(), 0, "Move count unchanged");
    }

    #[test]
    fn test_redo_with_no_future_fails() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);

        history.make_move(e2e4);
        let result = history.redo();

        assert!(!result, "Redo should fail with no future history");
        assert_eq!(history.move_count(), 1, "Move count unchanged");
    }

    // =============================================================================
    // Multiple Move Sequences
    // =============================================================================

    #[test]
    fn test_multiple_moves() {
        let mut history = GameHistory::new();

        // 1. e4 e5
        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));

        assert_eq!(history.move_count(), 2, "Should have 2 moves");
        assert!(history.can_undo(), "Should be able to undo");
        assert!(!history.can_redo(), "No future to redo");
    }

    #[test]
    fn test_multiple_undos() {
        let mut history = GameHistory::new();
        let initial_board = history.current_board().clone();

        // Make 3 moves
        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));

        assert_eq!(history.move_count(), 3, "Should have 3 moves");

        // Undo all moves
        assert!(history.undo(), "First undo should succeed");
        assert_eq!(history.move_count(), 2, "Should have 2 moves");

        assert!(history.undo(), "Second undo should succeed");
        assert_eq!(history.move_count(), 1, "Should have 1 move");

        assert!(history.undo(), "Third undo should succeed");
        assert_eq!(history.move_count(), 0, "Should have 0 moves");

        assert!(!history.undo(), "Fourth undo should fail");
        assert_eq!(
            history.current_board(),
            &initial_board,
            "Should be back at start"
        );
    }

    #[test]
    fn test_multiple_redos() {
        let mut history = GameHistory::new();

        // Make 3 moves and undo them all
        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));
        let final_board = history.current_board().clone();

        history.undo();
        history.undo();
        history.undo();

        // Redo all moves
        assert!(history.redo(), "First redo should succeed");
        assert_eq!(history.move_count(), 1, "Should have 1 move");

        assert!(history.redo(), "Second redo should succeed");
        assert_eq!(history.move_count(), 2, "Should have 2 moves");

        assert!(history.redo(), "Third redo should succeed");
        assert_eq!(history.move_count(), 3, "Should have 3 moves");

        assert!(!history.redo(), "Fourth redo should fail");
        assert_eq!(
            history.current_board(),
            &final_board,
            "Should be back at final position"
        );
    }

    // =============================================================================
    // Branching History (new move clears redo stack)
    // =============================================================================

    #[test]
    fn test_new_move_clears_redo_history() {
        let mut history = GameHistory::new();

        // Make 2 moves
        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));

        // Undo one move
        history.undo();
        assert_eq!(history.move_count(), 1, "Should have 1 move");
        assert!(history.can_redo(), "Should be able to redo");

        // Make a different move (branching)
        history.make_move(create_move(Square::D2, Square::D4));

        assert_eq!(history.move_count(), 2, "Should have 2 moves");
        assert!(history.can_undo(), "Should be able to undo");
        assert!(!history.can_redo(), "Redo should be cleared");

        // Verify we can't redo the old e5 move
        assert!(!history.redo(), "Redo should fail - history was cleared");
    }

    #[test]
    fn test_branching_preserves_earlier_history() {
        let mut history = GameHistory::new();

        // 1. e4 e5 2. Nf3
        history.make_move(create_move(Square::E2, Square::E4));
        let board_after_e4 = history.current_board().clone();
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));

        // Undo back to after 1. e4
        history.undo(); // Undo Nf3
        history.undo(); // Undo e5

        assert_eq!(history.move_count(), 1, "Should be at 1 move");
        assert_eq!(
            history.current_board(),
            &board_after_e4,
            "Should be after e4"
        );

        // Make different second move
        history.make_move(create_move(Square::C7, Square::C5)); // Sicilian!

        assert_eq!(history.move_count(), 2, "Should have 2 moves");

        // Can still undo back to e4
        history.undo();
        assert_eq!(
            history.current_board(),
            &board_after_e4,
            "Should be back after e4"
        );

        // Can undo to start
        history.undo();
        assert_eq!(history.move_count(), 0, "Should be at start");
    }

    // =============================================================================
    // Move Retrieval Tests
    // =============================================================================

    #[test]
    fn test_get_move_by_index() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);
        let e7e5 = create_move(Square::E7, Square::E5);

        history.make_move(e2e4);
        history.make_move(e7e5);

        assert_eq!(history.get_move(0), Some(&e2e4), "First move should be e2e4");
        assert_eq!(history.get_move(1), Some(&e7e5), "Second move should be e7e5");
        assert_eq!(history.get_move(2), None, "Index 2 should be out of bounds");
    }

    #[test]
    fn test_get_move_after_undo() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);
        let e7e5 = create_move(Square::E7, Square::E5);

        history.make_move(e2e4);
        history.make_move(e7e5);
        history.undo(); // Undo e7e5

        assert_eq!(history.move_count(), 1, "Should have 1 move");
        assert_eq!(history.get_move(0), Some(&e2e4), "First move still accessible");
        assert_eq!(
            history.get_move(1),
            None,
            "Second move not accessible after undo (FIXED)"
        );
    }

    #[test]
    fn test_get_move_empty_history() {
        let history = GameHistory::new();

        assert_eq!(history.get_move(0), None, "No moves in empty history");
    }

    // =============================================================================
    // State Consistency Tests
    // =============================================================================

    #[test]
    fn test_board_state_consistency_after_undo_redo() {
        let mut history = GameHistory::new();

        // Make a move and capture the board state
        history.make_move(create_move(Square::E2, Square::E4));
        let board_after_e4 = history.current_board().clone();

        // Undo and redo multiple times
        for _ in 0..5 {
            history.undo();
            history.redo();
        }

        assert_eq!(
            history.current_board(),
            &board_after_e4,
            "Board should be consistent after multiple undo/redo cycles"
        );
    }

    #[test]
    fn test_move_count_consistency() {
        let mut history = GameHistory::new();

        // Build up history with valid alternating moves
        let moves = vec![
            (Square::E2, Square::E4),  // 1. e4
            (Square::E7, Square::E5),  // 1... e5
            (Square::G1, Square::F3),  // 2. Nf3
            (Square::B8, Square::C7),  // 2... Nc6
            (Square::F1, Square::C4),  // 3. Bc4
        ];

        for (i, (from, to)) in moves.iter().enumerate() {
            history.make_move(create_move(*from, *to));
            assert_eq!(history.move_count(), i + 1, "Move count should increment");
        }

        // Tear down history
        for i in (0..moves.len()).rev() {
            history.undo();
            assert_eq!(history.move_count(), i, "Move count should decrement");
        }
    }

    #[test]
    fn test_can_undo_can_redo_consistency() {
        let mut history = GameHistory::new();

        // At start: can't undo or redo
        assert!(!history.can_undo() && !history.can_redo());

        // After move: can undo, can't redo
        history.make_move(create_move(Square::E2, Square::E4));
        assert!(history.can_undo() && !history.can_redo());

        // After undo: can't undo, can redo
        history.undo();
        assert!(!history.can_undo() && history.can_redo());

        // After redo: can undo, can't redo
        history.redo();
        assert!(history.can_undo() && !history.can_redo());
    }

    // =============================================================================
    // Complex Scenario Tests
    // =============================================================================

    #[test]
    fn test_complex_undo_redo_sequence() {
        let mut history = GameHistory::new();

        // Make 4 moves
        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));
        history.make_move(create_move(Square::B8, Square::C6));
        assert_eq!(history.move_count(), 4);

        // Undo 2 moves
        history.undo();
        history.undo();
        assert_eq!(history.move_count(), 2);

        // Redo 1 move
        history.redo();
        assert_eq!(history.move_count(), 3);

        // Make new move (clears remaining redo)
        history.make_move(create_move(Square::F8, Square::C5));
        assert_eq!(history.move_count(), 4);
        assert!(!history.can_redo(), "Redo should be cleared");

        // Undo all the way back
        history.undo();
        history.undo();
        history.undo();
        history.undo();
        assert_eq!(history.move_count(), 0);
        assert_eq!(history.current_board(), &Board::default());
    }

    #[test]
    fn test_undo_redo_with_promotions() {
        // Create a position where pawn can promote
        // Position: white pawn on e7, black king on a8, white king on a1        let mut history = GameHistory::new();
        let fen = "k7/4P3/8/8/8/8/8/K7 w - - 0 1";
        let board = Board::from_str(fen).expect("Valid fen");

        let mut history = GameHistory::new();
        // Manually set the starting position
        history.positions[0] = board;

        // Create a promotion move (e7-e8=Q)
        let promotion = ChessMove::new(
            Square::E7,
            Square::E8,
            Some(chess::Piece::Queen),
        );

        // Note: This will create an invalid position from the initial board,
        // but it tests the data structure's handling of promotion moves
        history.make_move(promotion);
        assert_eq!(history.move_count(), 1);

        // Verify the move is stored correctly
        if let Some(stored_move) = history.get_move(0) {
            assert_eq!(stored_move.get_promotion(), Some(chess::Piece::Queen));
        } else {
            panic!("Move should be retrievable");
        }

        // Undo and verify
        history.undo();
        assert_eq!(history.move_count(), 0);

        // Redo and verify
        history.redo();
        assert_eq!(history.move_count(), 1);

        // Verify promotion piece is still correct
        if let Some(stored_move) = history.get_move(0) {
            assert_eq!(stored_move.get_promotion(), Some(chess::Piece::Queen));
        }
    }

    // =============================================================================
    // Boundary and Edge Cases
    // =============================================================================

    #[test]
    fn test_alternating_undo_redo() {
        let mut history = GameHistory::new();
        history.make_move(create_move(Square::E2, Square::E4));

        // Alternate undo/redo 10 times
        for _ in 0..10 {
            assert!(history.undo());
            assert!(history.redo());
        }

        assert_eq!(history.move_count(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_max_reasonable_history() {
        let mut history = GameHistory::new();

        // Make a valid sequence of opening moves
        // This is a simple opening that stays legal
        let moves = vec![
            (Square::E2, Square::E4),  // 1. e4
            (Square::E7, Square::E5),  // 1... e5
            (Square::G1, Square::F3),  // 2. Nf3
            (Square::B8, Square::C6),  // 2... Nc6
            (Square::F1, Square::C4),  // 3. Bc4
            (Square::F8, Square::C5),  // 3... Bc5
            (Square::D2, Square::D3),  // 4. d3
            (Square::G8, Square::F6),  // 4... Nf6
            (Square::B1, Square::C3),  // 5. Nc3
            (Square::D7, Square::D6),  // 5... d6
        ];

        // Make 10 moves
        for (from, to) in &moves {
            history.make_move(create_move(*from, *to));
        }

        assert_eq!(history.move_count(), 10, "Should have 10 moves");

        // Undo all
        for i in (0..10).rev() {
            assert!(history.undo(), "Undo {} should succeed", i);
            assert_eq!(history.move_count(), i, "Move count should be {}", i);
        }

        assert_eq!(history.move_count(), 0);
        assert_eq!(history.current_board(), &Board::default());

        // Verify we can redo them all back
        for i in 0..10 {
            assert!(history.redo(), "Redo {} should succeed", i);
            assert_eq!(history.move_count(), i + 1, "Move count should be {}", i + 1);
        }

        assert_eq!(history.move_count(), 10);
    }

    // Verify current_moves() helper
    #[test]
    fn test_current_moves_respects_undo() {
        let mut history = GameHistory::new();

        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));

        assert_eq!(history.current_moves().len(), 3);

        history.undo();
        assert_eq!(history.current_moves().len(), 2, "Should only show 2 moves after undo");

        history.undo();
        assert_eq!(history.current_moves().len(), 1, "Should only show 1 move");

        history.redo();
        assert_eq!(history.current_moves().len(), 2, "Should show 2 moves after redo");
    }
}