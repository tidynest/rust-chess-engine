//! The UI side of the engine link, plus the move and history operations
//! that have to keep it informed.

use chess::{Board, ChessMove, Color as ChessColor, Piece as ChessPiece, Square as ChessSquare};
use chess_core::{GameHistory, notation};
use chess_engine::{EngineResponse, SearchLimit};
use std::str::FromStr;
use std::time::{Duration, Instant};

use super::engine_link::{EngineCommand, EngineEvent, EngineStatus, SearchRequest};
use super::state::{ChessApp, EngineLine};

/// Engine operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineMode {
    /// Fixed search depth
    Depth,
    /// Time-limited search
    TimeLimit,
}

/// Why a search was started: to play its result, or only to show it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchKind {
    Play,
    Analyse,
}

impl ChessApp {
    /// Take in everything the engine thread sent since the last frame.
    /// Returns the best move of the current search once it has arrived.
    pub(crate) fn poll_engine_responses(&mut self) -> Option<String> {
        let events: Vec<EngineEvent> = self.engine_rx.as_ref()?.try_iter().collect();

        let mut best_move = None;
        for event in events {
            match event {
                EngineEvent::Ready => {
                    self.engine_status = EngineStatus::Ready;
                    self.send_engine_options();
                }
                EngineEvent::Failed(message) => {
                    self.engine_status = EngineStatus::Failed(message);
                    self.engine_thinking = false;
                    self.play_vs_computer = false;
                }
                EngineEvent::Error(message) => self.notice = Some(format!("Engine: {message}")),
                // A reply to a position the user has already left.
                EngineEvent::Search { id, .. } if id != self.search_id => {}
                EngineEvent::Search {
                    response:
                        EngineResponse::Info {
                            depth,
                            multipv,
                            score,
                            nodes,
                            pv,
                            ..
                        },
                    ..
                } => {
                    // Stockfish scores from the side to move; the UI shows White's view.
                    let black_to_move =
                        self.game_history.current_board().side_to_move() == ChessColor::Black;
                    let line = EngineLine {
                        depth,
                        score: if black_to_move {
                            score.flipped()
                        } else {
                            score
                        },
                        pv,
                    };
                    // Lines arrive in order, so a gap can only be a line past
                    // the number asked for.
                    let index = multipv.saturating_sub(1) as usize;
                    if let Some(slot) = self.engine_lines.get_mut(index) {
                        *slot = line;
                    } else if index == self.engine_lines.len() {
                        self.engine_lines.push(line);
                    }
                    self.engine_nodes = nodes;
                }
                EngineEvent::Search {
                    response: EngineResponse::BestMove { mv, .. },
                    ..
                } => {
                    self.engine_thinking = false;
                    if self.search_kind == SearchKind::Analyse {
                        self.analysis_complete = true;
                        continue;
                    }
                    // The line's first move is about to be played; keep the
                    // continuation so it still formats from the new position.
                    if let Some(first) = self.engine_lines.first_mut()
                        && mv.is_some()
                        && first.pv.first() == mv.as_ref()
                    {
                        first.pv.remove(0);
                    }
                    best_move = mv;
                }
                EngineEvent::Search { .. } => {}
            }
        }
        best_move
    }

    /// Start whichever search the position calls for: the computer's move
    /// when it is its turn on the live line, otherwise an analysis of the
    /// position on screen if that is switched on and not done yet.
    pub(crate) fn auto_request(&mut self) {
        if self.engine_thinking || self.engine_status != EngineStatus::Ready || self.is_game_over()
        {
            return;
        }
        if self.computer_to_move() && !self.disable_auto_request {
            self.start_search(SearchKind::Play);
        } else if self.analysis && !self.analysis_complete {
            self.start_search(SearchKind::Analyse);
        }
    }

    fn start_search(&mut self, kind: SearchKind) {
        // On a clock the engine manages its own time; otherwise the fixed limit.
        let limit = match (kind, &self.clock) {
            (SearchKind::Play, Some(clock)) => {
                let inc = clock.increment().as_millis() as u64;
                SearchLimit::Clock {
                    wtime: clock.remaining(ChessColor::White).as_millis() as u64,
                    btime: clock.remaining(ChessColor::Black).as_millis() as u64,
                    winc: inc,
                    binc: inc,
                }
            }
            _ => match self.engine_mode {
                EngineMode::Depth => SearchLimit::Depth(self.engine_depth),
                EngineMode::TimeLimit => {
                    SearchLimit::MoveTime(self.engine_movetime.unwrap_or(1000))
                }
            },
        };
        let lines = match kind {
            SearchKind::Play => 1,
            SearchKind::Analyse => self.analysis_lines,
        };
        if lines != self.engine_multipv {
            self.send(EngineCommand::SetOption {
                name: "MultiPV".to_owned(),
                value: lines.to_string(),
            });
            self.engine_multipv = lines;
        }
        // Lines beyond the first belong to the position searched before.
        self.engine_lines.truncate(1);
        self.search_id += 1;
        let request = SearchRequest {
            id: self.search_id,
            position: self.uci_position(),
            limit,
            // Analysis is always at full strength; the skill level shapes play only.
            skill_level: match kind {
                SearchKind::Play => self.engine_skill_level,
                SearchKind::Analyse => 20,
            },
        };
        self.search_kind = kind;
        self.engine_thinking = self.send(EngineCommand::Search(request));
    }

    /// True while a search whose result will be played is running; board
    /// input waits for it. Analysis never blocks the board.
    pub(crate) fn waiting_for_engine_move(&self) -> bool {
        self.engine_thinking && self.search_kind == SearchKind::Play
    }

    /// True when the engine panel and the eval bar have something to show.
    pub(crate) fn engine_in_use(&self) -> bool {
        self.play_vs_computer || self.analysis
    }

    /// Mate, a draw, a flag fall or a resignation.
    pub(crate) fn is_game_over(&self) -> bool {
        self.timeout.is_some() || self.resigned.is_some() || self.game_history.is_over()
    }

    /// The side to move gives up; against the computer that is the human.
    pub(crate) fn resign(&mut self) {
        if self.is_game_over() {
            return;
        }
        let loser = if self.play_vs_computer {
            !self.computer_color
        } else {
            self.board().side_to_move()
        };
        self.resigned = Some(loser);
        self.abort_search();
    }

    /// The result for the PGN, including a loss on time or by resignation.
    pub(crate) fn result(&self) -> &'static str {
        match self.timeout.or(self.resigned) {
            Some(ChessColor::White) => "0-1",
            Some(ChessColor::Black) => "1-0",
            None => self.game_history.result(),
        }
    }

    /// Charge the side to move, stop the game when its flag falls, and keep
    /// the display moving while the clock runs. Paused while browsing the
    /// history and once the game is over.
    pub(crate) fn tick_clock(&mut self, ctx: &egui::Context) {
        let paused = self.is_game_over() || self.game_history.can_redo();
        let side = self.board().side_to_move();
        let Some(clock) = &mut self.clock else {
            return;
        };
        if paused {
            clock.pause();
            return;
        }
        let flagged = clock.tick(side, Instant::now());
        let running = clock.is_running();
        if flagged {
            self.timeout = Some(side);
            self.abort_search();
        } else if running {
            ctx.request_repaint_after(Duration::from_millis(100));
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
                self.notice = Some(format!("Engine played {move_str}, which is not legal here"));
                // Without this the next frame would ask again, forever.
                self.disable_auto_request = true;
            }
        }
    }

    /// Play a legal move on the current position, whoever chose it.
    pub(crate) fn play_move(&mut self, mv: ChessMove) {
        let mover = self.board().side_to_move();
        self.game_history.make_move(mv);
        let now = Instant::now();
        if let Some(clock) = &mut self.clock {
            clock.press(mover, now, self.game_history.move_count());
        }
        self.animation = Some((mv, now));
        self.disable_auto_request = false;
        self.position_changed();
    }

    /// Drop any selection made on the old position and any search still
    /// running on it.
    pub(crate) fn position_changed(&mut self) {
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
        self.pending_promotion = None;
        self.analysis_complete = false;
        self.notice = None;
        self.abort_search();
    }

    /// Put the clock back to where it stood at the current ply, after the
    /// history moved under it.
    fn sync_clock(&mut self) {
        let ply = self.game_history.move_count();
        if let Some(clock) = &mut self.clock {
            clock.restore(ply);
        }
    }

    /// True in a game against the computer when it is the computer's turn.
    pub(crate) fn computer_to_move(&self) -> bool {
        self.play_vs_computer
            && self.game_history.current_board().side_to_move() == self.computer_color
    }

    /// Take back one ply, or two against the computer so the human is to move again.
    pub(crate) fn undo(&mut self) {
        self.step_history(GameHistory::undo, false);
    }

    /// Replay one ply, or two against the computer so the human is to move again.
    pub(crate) fn redo(&mut self) {
        self.step_history(GameHistory::redo, true);
    }

    fn step_history(&mut self, step: fn(&mut GameHistory) -> bool, forward: bool) {
        if !step(&mut self.game_history) {
            return;
        }
        if self.computer_to_move() {
            step(&mut self.game_history);
        }
        self.position_changed();
        // Still the computer's turn means we hit an end of the history; let it play.
        self.disable_auto_request = false;
        self.sync_clock();

        // Slide the piece of the last ply stepped; on undo, back to where it came from.
        let ply = self.game_history.move_count();
        let history = &self.game_history;
        let stepped = if forward {
            ply.checked_sub(1)
                .and_then(|index| history.get_move(index).copied())
        } else {
            history
                .get_move(ply)
                .map(|mv| ChessMove::new(mv.get_dest(), mv.get_source(), None))
        };
        self.animation = stepped.map(|mv| (mv, Instant::now()));
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

    /// Show the position after `ply` moves. The engine stays quiet unless
    /// that is the live end of the history.
    pub(crate) fn jump_to_ply(&mut self, ply: usize) {
        let current = self.game_history.move_count();
        for _ in ply..current {
            self.game_history.undo();
        }
        for _ in current..ply {
            self.game_history.redo();
        }
        self.position_changed();
        self.disable_auto_request = self.game_history.can_redo();
        self.sync_clock();
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
    fn test_undo_gives_the_clock_time_back() {
        use crate::app::clock::Clock;

        let mut app = ChessApp::headless();
        app.clock = Some(Clock::new(5, 0));
        play(&mut app, &["e2e4"]);
        let clock = app.clock.as_mut().unwrap();
        clock.tick(ChessColor::Black, Instant::now() + Duration::from_secs(30));
        assert!(clock.remaining(ChessColor::Black) <= Duration::from_secs(270));

        app.undo();
        let clock = app.clock.as_ref().unwrap();
        assert_eq!(clock.remaining(ChessColor::Black), Duration::from_secs(300));
        assert!(!clock.is_running());
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
    fn test_lines_are_kept_by_number_and_cut_on_a_new_search() {
        let mut app = ChessApp::headless();
        let (tx, rx) = std::sync::mpsc::channel();
        app.engine_rx = Some(rx);
        app.search_id = 1;
        let info = |multipv, cp, pv: &str| EngineEvent::Search {
            id: 1,
            response: EngineResponse::Info {
                depth: 10,
                multipv,
                score: chess_engine::Score::Cp(cp),
                nodes: 0,
                nps: 0,
                pv: vec![pv.to_owned()],
            },
        };
        tx.send(info(1, 30, "e2e4")).unwrap();
        tx.send(info(2, 20, "d2d4")).unwrap();
        // A line with a gap before it cannot be placed and is dropped.
        tx.send(info(4, 10, "c2c4")).unwrap();
        tx.send(info(1, 35, "e2e4")).unwrap();
        app.poll_engine_responses();
        assert_eq!(app.engine_lines.len(), 2);
        assert_eq!(app.engine_evaluation(), Some(chess_engine::Score::Cp(35)));
        assert_eq!(app.engine_lines[1].pv, ["d2d4"]);

        app.analysis_lines = 3;
        app.start_search(SearchKind::Analyse);
        assert_eq!(app.engine_multipv, 3);
        assert_eq!(app.engine_lines.len(), 1, "only the first line survives");
        app.start_search(SearchKind::Play);
        assert_eq!(app.engine_multipv, 1);
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
