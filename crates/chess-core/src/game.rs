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
        self.moves.get(index)
    }
}

impl Default for GameHistory {
    fn default() -> Self {
        Self::new()
    }
}