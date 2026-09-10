//! The UI side of the engine link, plus the move and history operations
//! that have to keep it informed.

use chess::{
    Board, BoardStatus, ChessMove, Color as ChessColor, Piece as ChessPiece, Square as ChessSquare,
};
use chess_core::{GameHistory, notation};
use chess_engine::EngineResponse;
use std::str::FromStr;

use super::engine_link::{EngineCommand, EngineEvent, EngineStatus, SearchRequest};
use super::state::ChessApp;

/// Engine operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineMode {
    /// Fixed search depth
    Depth,
    /// Time-limited search
    TimeLimit,
}

impl ChessApp {
    /// Take in everything the engine thread sent since the last frame.
    /// Returns the best move of the current search once it has arrived.
    pub(crate) fn poll_engine_responses(&mut self) -> Option<String> {
        let events: Vec<EngineEvent> = self.engine_rx.as_ref()?.try_iter().collect();

        let mut best_move = None;
        for event in events {
            match event {
                EngineEvent::Ready => self.engine_status = EngineStatus::Ready,
                EngineEvent::Failed(message) => {
                    self.engine_status = EngineStatus::Failed(message);
                    self.engine_thinking = false;
                    self.play_vs_computer = false;
                }
                // A reply to a position the user has already left.
                EngineEvent::Search { id, .. } if id != self.search_id => {}
                EngineEvent::Search {
                    response:
                        EngineResponse::Info {
                            depth,
                            score,
                            nodes,
                            pv,
                            ..
                        },
                    ..
                } => {
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
                }
                EngineEvent::Search {
                    response: EngineResponse::BestMove { mv, .. },
                    ..
                } => {
                    self.engine_thinking = false;
                    best_move = mv;
                }
                EngineEvent::Search { .. } => {}
            }
        }
        best_move
    }

    /// Ask the engine for a move in the current position.
    pub(crate) fn request_engine_move(&mut self) {
        if self.engine_thinking
            || self.engine_status != EngineStatus::Ready
            || !self.computer_to_move()
            || self.board().status() != BoardStatus::Ongoing
        {
            return;
        }

        self.search_id += 1;
        let request = SearchRequest {
            id: self.search_id,
            position: self.uci_position(),
            depth: (self.engine_mode == EngineMode::Depth).then_some(self.engine_depth),
            movetime: (self.engine_mode == EngineMode::TimeLimit)
                .then_some(self.engine_movetime)
                .flatten(),
            skill_level: self.engine_skill_level,
        };
        self.engine_thinking = self.send(EngineCommand::Search(request));
    }

    /// Auto-request engine move if conditions are met
    pub(crate) fn auto_request_engine_move(&mut self) {
        if !self.disable_auto_request {
            self.request_engine_move();
        }
    }

    /// Forget the running search. Its replies carry the old id and are dropped.
    pub(crate) fn abort_search(&mut self) {
        self.search_id += 1;
        if self.engine_thinking {
            self.engine_thinking = false;
            self.send(EngineCommand::Stop);
        }
    }

    /// Tell the engine a new game starts; returns nothing because a missing
    /// engine is reported through `engine_status`, not per command.
    pub(crate) fn send(&self, command: EngineCommand) -> bool {
        self.engine_tx
            .as_ref()
            .is_some_and(|tx| tx.send(command).is_ok())
    }

    /// The `position` argument for the current line: the start position and
    /// every move played, so the engine can see repetitions and the 50-move
    /// clock, which a bare FEN from the `chess` crate does not carry.
    fn uci_position(&self) -> String {
        let start = self.game_history.start_board();
        let mut position = if *start == Board::default() {
            "startpos".to_owned()
        } else {
            format!("fen {start}")
        };
        let moves = self.game_history.current_moves();
        if !moves.is_empty() {
            position.push_str(" moves");
            for mv in moves {
                position.push(' ');
                position.push_str(&mv.to_string());
            }
        }
        position
    }

    /// Apply engine's move to the game
    pub(crate) fn apply_engine_move(&mut self, move_str: &str) {
        match self.parse_uci_move(move_str, self.game_history.current_board()) {
            Some(mv) => self.play_move(mv),
            None => {
                eprintln!("engine played {move_str}, which is not legal here");
                // Without this the next frame would ask again, forever.
                self.disable_auto_request = true;
            }
        }
    }

    /// Play a legal move on the current position, whoever chose it.
    pub(crate) fn play_move(&mut self, mv: ChessMove) {
        self.game_history.make_move(mv);
        self.last_move = Some((mv.get_source(), mv.get_dest()));
        self.disable_auto_request = false;
        self.position_changed();
    }

    /// Drop any selection made on the old position and any search still
    /// running on it.
    pub(crate) fn position_changed(&mut self) {
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
        self.pending_promotion = None;
        self.abort_search();
    }

    /// True in a game against the computer when it is the computer's turn.
    pub(crate) fn computer_to_move(&self) -> bool {
        self.play_vs_computer
            && self.game_history.current_board().side_to_move() == self.computer_color
    }

    /// Take back one ply, or two against the computer so the human is to move again.
    pub(crate) fn undo(&mut self) {
        self.step_history(GameHistory::undo);
    }

    /// Replay one ply, or two against the computer so the human is to move again.
    pub(crate) fn redo(&mut self) {
        self.step_history(GameHistory::redo);
    }

    fn step_history(&mut self, step: fn(&mut GameHistory) -> bool) {
        if !step(&mut self.game_history) {
            return;
        }
        if self.computer_to_move() {
            step(&mut self.game_history);
        }
        self.position_changed();
        // Still the computer's turn means we hit an end of the history; let it play.
        self.disable_auto_request = false;
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
        self.position_changed();
        self.disable_auto_request = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::Board;
    use chess_core::GameHistory;

    fn play(app: &mut ChessApp, moves: &[&str]) {
        for mv in moves {
            let mv = app
                .parse_uci_move(mv, app.game_history.current_board())
                .unwrap();
            app.play_move(mv);
        }
    }

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
    fn test_uci_position_lists_the_moves_played() {
        let mut app = ChessApp::headless();
        assert_eq!(app.uci_position(), "startpos");

        play(&mut app, &["e2e4", "e7e5", "g1f3"]);
        assert_eq!(app.uci_position(), "startpos moves e2e4 e7e5 g1f3");

        app.game_history.undo();
        assert_eq!(app.uci_position(), "startpos moves e2e4 e7e5");

        let fen = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1";
        app.game_history = GameHistory::from_board(Board::from_str(fen).unwrap());
        play(&mut app, &["a7a8n"]);
        assert_eq!(app.uci_position(), format!("fen {fen} moves a7a8n"));
    }

    #[test]
    fn test_undo_against_computer_lands_on_human_turn() {
        let mut app = ChessApp::headless();
        app.play_vs_computer = true;
        app.computer_color = ChessColor::Black;
        play(&mut app, &["e2e4", "e7e5", "g1f3"]);

        // Nf3 has no reply yet, so only that ply comes back.
        app.undo();
        assert_eq!(app.game_history.move_count(), 2);

        // Undoing e5 lands on the computer's turn, so e4 comes back too.
        app.undo();
        assert_eq!(app.game_history.move_count(), 0);

        // Redo replays e4 and, since that is the computer's turn, e5 as well.
        app.redo();
        assert_eq!(app.game_history.move_count(), 2);
        assert!(!app.disable_auto_request);
    }

    #[test]
    fn test_stale_replies_are_dropped() {
        let mut app = ChessApp::headless();
        let (tx, rx) = std::sync::mpsc::channel();
        app.engine_rx = Some(rx);
        app.search_id = 7;
        app.engine_thinking = true;

        let best = |id| EngineEvent::Search {
            id,
            response: EngineResponse::BestMove {
                mv: Some("e2e4".into()),
                ponder: None,
            },
        };
        tx.send(best(6)).unwrap();
        assert_eq!(app.poll_engine_responses(), None);
        assert!(app.engine_thinking);

        tx.send(best(7)).unwrap();
        assert_eq!(app.poll_engine_responses(), Some("e2e4".to_owned()));
        assert!(!app.engine_thinking);
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
