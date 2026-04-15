//! Engine communication and move management.
//!
//! Handles Stockfish engine communication, move parsing, and game history synchronization.

use chess::{ChessMove, Color as ChessColor, Piece as ChessPiece, Square as ChessSquare};
use chess_core::{Color, GameHistory, notation};
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
                        nps: _,
                        pv,
                    }) => {
                        eprintln!("Engine info: depth={}, score={}", depth, score);

                        self.engine_depth_current = depth;

                        let board = self.game_history.current_board();
                        let adjusted_score = if board.side_to_move() == ChessColor::Black {
                            -score
                        } else {
                            score
                        };

                        self.engine_evaluation = Some(adjusted_score as f32 / 100.0);
                        self.engine_nodes = nodes;
                        self.engine_pv = pv;

                        response_count += 1;
                    }
                    Ok(EngineResponse::BestMove { mv, .. }) => {
                        best_move_to_apply = Some(mv);
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

        if self.move_history.len() == self.last_move_count_check {
            self.loop_protection_counter += 1;
            if self.loop_protection_counter > 3 {
                eprintln!("ERROR: Move count stuck! Breaking loop.");
                self.play_vs_computer = false;
                self.loop_protection_counter = 0;
                return;
            }
        } else {
            self.loop_protection_counter = 0;
            self.last_move_count_check = self.move_history.len();
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
        let chess_move = match self.parse_uci_move(move_str, self.game_history.current_board()) {
            Some(m) => m,
            None => {
                eprintln!("ERROR: Failed to parse move: {}", move_str);
                return;
            }
        };

        let san = notation::format_move_san(&chess_move, self.game_history.current_board());

        self.game_history.make_move(chess_move);
        self.move_history.push(san);
        self.last_move = Some((chess_move.get_source(), chess_move.get_dest()));

        self.sync_engine_from_history();

        self.selected_square = None;
        self.legal_moves_for_selected.clear();
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

    /// Synchronize engine state with game history
    pub(crate) fn sync_engine_from_history(&mut self) {
        let fen = self.game_history.current_board().to_string();
        self.engine = chess_core::ChessEngine::from_fen(&fen)
            .unwrap_or_else(|_| chess_core::ChessEngine::new());

        if self.skip_history_rebuild {
            self.skip_history_rebuild = false;
            return;
        }

        self.move_history.clear();
        for i in 0..self.game_history.move_count() {
            if let Some(chess_move) = self.game_history.get_move(i) {
                let mut temp_history = GameHistory::new();
                for j in 0..i {
                    if let Some(prev_move) = self.game_history.get_move(j) {
                        temp_history.make_move(*prev_move);
                    }
                }
                let san = notation::format_move_san(chess_move, temp_history.current_board());
                self.move_history.push(san);
            }
        }
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

    /// Jump to specific move in history
    pub(crate) fn jump_to_move(&mut self, target_index: usize) {
        let current_index = self.game_history.move_count();

        if target_index + 1 == current_index {
            return;
        }

        if target_index + 1 < current_index {
            while self.game_history.move_count() > target_index + 1 {
                self.game_history.undo();
            }
        } else {
            while self.game_history.move_count() < target_index + 1 {
                self.game_history.redo();
            }
        }

        self.skip_history_rebuild = true;
        self.sync_engine_from_history();
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
        self.disable_auto_request = true;
        self.viewing_move_index = Some(target_index);
    }
}
