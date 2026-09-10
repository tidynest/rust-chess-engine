//! Engine communication and move management.
//!
//! Handles Stockfish engine communication, move parsing, and game history synchronization.

use chess::{ChessMove, Color as ChessColor, Piece as ChessPiece, Square as ChessSquare};
use chess_core::{ChessEngine, Color, notation};
use chess_engine::{EngineCommand, EngineResponse};
use std::str::FromStr;
use std::sync::mpsc::TryRecvError;

use super::state::ChessApp;
use chess_core::GameState;

/// Engine operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineMode {
    /// Fixed search depth
    Depth,
    /// Time-limited search
    TimeLimit,
    /// Full strength (no limits)
    FullStrength,
}

impl ChessApp {
    /// Poll engine for responses and return best move if available
    pub(crate) fn poll_engine_responses(&mut self) -> Option<String> {
        let mut best_move_to_apply: Option<String> = None;

        if let Some(rx) = &self.stockfish_rx {
            let mut response_count = 0;

            loop {
                match rx.try_recv() {
                    Ok(EngineResponse::Info {
                        depth,
                        score,
                        nodes,
                        pv,
                        ..
                    }) => {
                        self.engine_depth_current = depth;

                        // Stockfish scores from the side to move; the UI shows White's view.
                        let black_to_move =
                            self.game_history.current_board().side_to_move() == ChessColor::Black;
                        self.engine_evaluation = Some(if black_to_move {
                            score.flipped()
                        } else {
                            score
                        });
                        self.engine_nodes = nodes;
                        self.engine_pv = pv;

                        response_count += 1;
                    }
                    Ok(EngineResponse::BestMove { mv, .. }) => {
                        best_move_to_apply = mv;
                        self.engine_thinking = false;
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        eprintln!("ERROR: Chess Engine thread disconnected");
                        self.engine_thinking = false;
                        break;
                    }
                    _ => {
                        response_count += 1;
                    }
                }

                if response_count > 100 {
                    eprintln!("WARNING: Too many engine responses in one frame!");
                    break;
                }
            }
        }

        best_move_to_apply
    }

    /// Request engine to calculate best move
    pub(crate) fn request_engine_move(&mut self) {
        if !self.play_vs_computer || self.engine_thinking {
            return;
        }

        if self.engine.is_checkmate() || self.engine.is_stalemate() {
            return;
        }

        if self.game_history.move_count() == self.last_move_count_check {
            self.loop_protection_counter += 1;
            if self.loop_protection_counter > 3 {
                eprintln!("ERROR: Move count stuck! Breaking loop.");
                self.play_vs_computer = false;
                self.loop_protection_counter = 0;
                return;
            }
        } else {
            self.loop_protection_counter = 0;
            self.last_move_count_check = self.game_history.move_count();
        }

        let current_turn = if self.engine.side_to_move() == Color::White {
            ChessColor::White
        } else {
            ChessColor::Black
        };

        if current_turn != self.computer_color {
            return;
        }

        self.engine_thinking = true;

        if let Some(tx) = &self.stockfish_tx {
            let fen = self.game_history.current_board().to_string();
            let result = tx.send(EngineCommand::GetBestMove {
                fen,
                depth: if self.engine_mode == EngineMode::Depth {
                    Some(self.engine_depth)
                } else {
                    None
                },
                movetime: if self.engine_mode == EngineMode::TimeLimit {
                    self.engine_movetime
                } else {
                    None
                },
                skill_level: self.engine_skill_level,
            });

            if result.is_err() {
                eprintln!("ERROR: Failed to send to engine");
                self.engine_thinking = false;
            }
        }
    }

    /// Apply engine's move to the game
    pub(crate) fn apply_engine_move(&mut self, move_str: &str) {
        match self.parse_uci_move(move_str, self.game_history.current_board()) {
            Some(mv) => self.play_move(mv),
            None => eprintln!("ERROR: Failed to parse move: {}", move_str),
        }
    }

    /// Play a legal move on the current position, whoever chose it.
    pub(crate) fn play_move(&mut self, mv: ChessMove) {
        self.game_history.make_move(mv);
        self.last_move = Some((mv.get_source(), mv.get_dest()));
        self.disable_auto_request = false;
        self.sync_engine();
    }

    /// Point the move validator at the history's current position and drop
    /// any selection made on the old one.
    pub(crate) fn sync_engine(&mut self) {
        self.engine = ChessEngine::from_board(*self.game_history.current_board());
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
        self.pending_promotion = None;
    }

    /// Auto-request engine move if conditions are met
    pub(crate) fn auto_request_engine_move(&mut self) {
        if self.play_vs_computer
            && !self.engine_thinking
            && !self.engine.is_checkmate()
            && !self.engine.is_stalemate()
            && !self.disable_auto_request
        {
            let current_turn = if self.engine.side_to_move() == Color::White {
                ChessColor::White
            } else {
                ChessColor::Black
            };

            if current_turn == self.computer_color {
                self.request_engine_move();
            }
        }
    }

    /// Parse UCI move string to ChessMove
    pub fn parse_uci_move(&self, move_str: &str, board: &chess::Board) -> Option<ChessMove> {
        if move_str.len() < 4 {
            return None;
        }

        let from_str = &move_str[0..2];
        let to_str = &move_str[2..4];

        let (from, to) = match (
            ChessSquare::from_str(from_str),
            ChessSquare::from_str(to_str),
        ) {
            (Ok(f), Ok(t)) => (f, t),
            _ => return None,
        };

        let promotion = if move_str.len() > 4 {
            match &move_str[4..5] {
                "q" => Some(ChessPiece::Queen),
                "r" => Some(ChessPiece::Rook),
                "b" => Some(ChessPiece::Bishop),
                "n" => Some(ChessPiece::Knight),
                _ => None,
            }
        } else {
            None
        };

        let mut legal_moves = chess::MoveGen::new_legal(board);
        legal_moves.find(|m| {
            m.get_source() == from && m.get_dest() == to && m.get_promotion() == promotion
        })
    }

    /// Format principal variation in SAN notation
    pub fn format_pv_san(&self, pv: &[String]) -> Vec<String> {
        let mut formatted = Vec::new();
        let mut temp_board = *self.game_history.current_board();

        for move_str in pv.iter().take(6) {
            if let Some(chess_move) = self.parse_uci_move(move_str, &temp_board) {
                let san = notation::format_move_san(&chess_move, &temp_board);
                formatted.push(san);
                temp_board = temp_board.make_move_new(chess_move);
            } else {
                break;
            }
        }

        formatted
    }

    /// Show the position after move `target_index` without the engine replying.
    pub(crate) fn jump_to_move(&mut self, target_index: usize) {
        let current = self.game_history.move_count();
        let target = target_index + 1;
        for _ in target..current {
            self.game_history.undo();
        }
        for _ in current..target {
            self.game_history.redo();
        }
        self.sync_engine();
        self.disable_auto_request = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::Board;
    use chess_core::GameHistory;

    #[test]
    fn test_parse_uci_move_basic() {
        let app = ChessApp::headless();
        let mv = app.parse_uci_move("e2e4", &Board::default()).unwrap();
        assert_eq!(mv.get_source().to_string(), "e2");
        assert_eq!(mv.get_dest().to_string(), "e4");
    }

    #[test]
    fn test_parse_uci_move_promotion() {
        let board = Board::from_str("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let app = ChessApp::headless();
        let mv = app.parse_uci_move("a7a8q", &board).unwrap();
        assert_eq!(mv.get_promotion(), Some(ChessPiece::Queen));
    }

    #[test]
    fn test_parse_uci_move_invalid() {
        let board = Board::default();
        let app = ChessApp::headless();
        assert!(app.parse_uci_move("e2e5", &board).is_none());
        assert!(app.parse_uci_move("e2", &board).is_none());
        assert!(app.parse_uci_move("xyz", &board).is_none());
    }

    #[test]
    fn test_format_pv_san_basic() {
        let app = ChessApp::headless();
        let pv = ["e2e4".to_string(), "e7e5".to_string(), "g1f3".to_string()];
        assert_eq!(app.format_pv_san(&pv), vec!["e4", "e5", "Nf3"]);
    }

    #[test]
    fn test_format_pv_san_with_capture() {
        let mut app = ChessApp::headless();
        let board =
            Board::from_str("r1bqkbnr/ppp2ppp/2n5/3pp3/3PP3/5N2/PPP2PPP/RNBQKB1R w KQkq d6 0 4")
                .unwrap();
        app.game_history = GameHistory::from_board(board);
        assert_eq!(app.format_pv_san(&["e4d5".to_string()]), vec!["exd5"]);
    }
}
