//! Application state management.
//!
//! Contains the ChessApp struct and all game state.

use chess::{Board, ChessMove, Color as ChessColor, Piece as ChessPiece, Square as ChessSquare};
use chess_core::{GameHistory, PgnTags};
use chess_engine::Score;
use eframe::egui::{Context, Pos2};
use std::sync::mpsc::{Receiver, channel};
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

use crate::ui::theme::{Theme, ThemeVariant};

use super::clock::Clock;
use super::engine_comm::{EngineMode, SearchKind};
use super::engine_link::{self, EngineCommand, EngineEvent, EngineStatus};
use super::games;
use super::settings::Settings;

/// Style for displaying captured pieces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapturedPiecesStyle {
    /// Show only the advantage (Lichess style)
    Lichess,
    /// Show all captured pieces for both sides (Chess.com style)
    ChessCom,
}

/// One line of the engine's search, its score from White's side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineLine {
    pub depth: u32,
    pub score: Score,
    pub pv: Vec<String>,
}

/// Main application state
pub struct ChessApp {
    // Game state
    pub game_history: GameHistory,
    pub selected_square: Option<ChessSquare>,
    pub legal_moves_for_selected: Vec<ChessMove>,
    /// A pawn move waiting for the player to choose the promotion piece.
    pub pending_promotion: Option<(ChessSquare, ChessSquare)>,
    /// Text of the "Set up position" window while it is open.
    pub fen_input: Option<String>,
    /// Text of the "Load PGN" window while it is open.
    pub pgn_input: Option<String>,
    /// A piece sliding along this move since this instant.
    pub animation: Option<(ChessMove, Instant)>,
    /// A line for the status panel: where a game was saved, or why not.
    pub notice: Option<String>,

    // UI state
    pub board_flip: bool,
    pub dragging_piece: Option<(ChessSquare, ChessPiece, ChessColor)>,
    pub drag_pos: Option<Pos2>,
    pub show_eval_bar: bool,
    pub captured_display_style: CapturedPiecesStyle,

    // Engine state
    pub play_vs_computer: bool,
    pub computer_color: ChessColor,
    /// Evaluate whatever position is on screen, whoever is to move.
    pub analysis: bool,
    /// The analysis of the current position has run to its limit.
    pub analysis_complete: bool,
    /// What the running or last search was for.
    pub search_kind: SearchKind,
    pub engine_tx: Option<UnboundedSender<EngineCommand>>,
    pub engine_rx: Option<Receiver<EngineEvent>>,
    pub engine_status: EngineStatus,
    /// Id of the latest search; replies to any other id are stale.
    pub search_id: u64,
    pub engine_thinking: bool,
    /// The lines of the running or last search, best first.
    pub engine_lines: Vec<EngineLine>,
    pub engine_nodes: u64,
    /// How many lines analysis asks for; play always asks for one.
    pub analysis_lines: u32,
    /// The MultiPV value the engine has, so it is only sent on change.
    pub engine_multipv: u32,
    pub engine_depth: u32,
    pub engine_movetime: Option<u64>,
    pub engine_mode: EngineMode,
    /// Play at this Elo through UCI_LimitStrength, or at full strength.
    pub engine_elo: Option<u32>,
    /// Stockfish's Threads and Hash options.
    pub engine_threads: usize,
    pub engine_hash_mb: u32,

    /// Set while the user browses the history, so the engine does not reply
    /// to a position that is not the live one.
    pub disable_auto_request: bool,

    // Clock
    /// The running game's clock, if the game was started with one.
    pub clock: Option<Clock>,
    /// Whose flag fell; the game is over until New Game.
    pub timeout: Option<ChessColor>,
    /// Who resigned; the game is over until New Game.
    pub resigned: Option<ChessColor>,
    /// The "Resign?" prompt is open.
    pub confirm_resign: bool,
    /// The time control for the next game.
    pub clock_enabled: bool,
    pub clock_minutes: u32,
    pub clock_increment_s: u32,

    // UI theme
    pub theme: Theme,
    pub theme_variant: ThemeVariant,
}

impl ChessApp {
    /// Create the app and start the Stockfish thread.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::headless();
        app.start_engine(&cc.egui_ctx);
        Settings::load().apply(&mut app);
        app.face_computer();
        app.theme.apply(&cc.egui_ctx);
        app
    }

    /// Start the Stockfish thread, or start it again after a failure. The
    /// old thread, if any, sees its command channel close and quits.
    pub fn start_engine(&mut self, ctx: &Context) {
        let (engine_tx, commands) = tokio::sync::mpsc::unbounded_channel();
        let (events, engine_rx) = channel();
        engine_link::spawn(commands, events, ctx.clone());
        self.engine_tx = Some(engine_tx);
        self.engine_rx = Some(engine_rx);
        self.engine_status = EngineStatus::Starting;
        self.engine_thinking = false;
    }

    /// Switch theme; the caller applies it to the egui context.
    pub fn set_theme(&mut self, variant: ThemeVariant) {
        self.theme_variant = variant;
        self.theme = variant.to_theme();
    }

    /// The app with no engine thread, for tests and for `new` to build on.
    pub fn headless() -> Self {
        Self {
            game_history: GameHistory::new(),
            selected_square: None,
            legal_moves_for_selected: Vec::new(),
            board_flip: false,
            pending_promotion: None,
            fen_input: None,
            pgn_input: None,
            animation: None,
            notice: None,
            disable_auto_request: false,
            clock: None,
            timeout: None,
            resigned: None,
            confirm_resign: false,
            clock_enabled: false,
            clock_minutes: 5,
            clock_increment_s: 3,
            dragging_piece: None,
            drag_pos: None,
            play_vs_computer: false,
            computer_color: ChessColor::Black,
            analysis: false,
            analysis_complete: false,
            search_kind: SearchKind::Play,
            engine_tx: None,
            engine_rx: None,
            engine_status: EngineStatus::Starting,
            search_id: 0,
            engine_thinking: false,
            engine_lines: Vec::new(),
            engine_nodes: 0,
            analysis_lines: 1,
            engine_multipv: 1,
            engine_depth: 20,
            engine_movetime: Some(1000),
            engine_mode: EngineMode::Depth,
            engine_elo: None,
            engine_threads: default_threads(),
            engine_hash_mb: 128,
            show_eval_bar: true,
            captured_display_style: CapturedPiecesStyle::Lichess,
            theme: Theme::default(),
            theme_variant: ThemeVariant::ClassicMonochrome,
        }
    }

    /// The position on the board right now.
    pub fn board(&self) -> &Board {
        self.game_history.current_board()
    }

    /// The squares of the move that made the position on screen.
    pub fn last_move(&self) -> Option<(ChessSquare, ChessSquare)> {
        let index = self.game_history.move_count().checked_sub(1)?;
        let mv = self.game_history.get_move(index)?;
        Some((mv.get_source(), mv.get_dest()))
    }

    /// The PGN of the game on screen: who plays which side, today, the result.
    pub fn pgn(&self) -> String {
        let (white, black) = match (self.play_vs_computer, self.computer_color) {
            (false, _) => ("?", "?"),
            (true, ChessColor::White) => ("Stockfish", "Human"),
            (true, ChessColor::Black) => ("Human", "Stockfish"),
        };
        let date = games::today();
        let tags = PgnTags {
            white,
            black,
            date: &date,
        };
        self.game_history.pgn_with_result(tags, self.result())
    }

    /// Write the game to the games directory and say where, or why not.
    pub fn save_game(&mut self) {
        if self.game_history.move_count() == 0 {
            return;
        }
        self.notice = Some(match games::save(&self.pgn()) {
            Ok(path) => format!("Saved to {}", path.display()),
            Err(e) => format!("Could not save the game: {e}"),
        });
    }

    /// The first line's score, from White's side.
    pub fn engine_evaluation(&self) -> Option<Score> {
        self.engine_lines.first().map(|line| line.score)
    }

    /// The piece on `square`, with its colour.
    pub fn piece_at(&self, square: ChessSquare) -> Option<(ChessPiece, ChessColor)> {
        let board = self.board();
        Some((board.piece_on(square)?, board.color_on(square)?))
    }

    /// Send the Threads and Hash options to the engine.
    pub fn send_engine_options(&self) {
        for (name, value) in [
            ("Threads", self.engine_threads.to_string()),
            ("Hash", self.engine_hash_mb.to_string()),
        ] {
            self.send(EngineCommand::SetOption {
                name: name.to_owned(),
                value,
            });
        }
    }

    /// Put the human's pieces at the bottom of the board.
    pub fn face_computer(&mut self) {
        self.board_flip = self.computer_color == ChessColor::White;
    }

    /// Reset the game to initial position
    pub fn new_game(&mut self) {
        self.start_game(GameHistory::new());
    }

    /// Replace the game with `history` and tell the engine to forget the old one.
    pub fn start_game(&mut self, history: GameHistory) {
        self.game_history = history;
        self.clock = self
            .clock_enabled
            .then(|| Clock::new(self.clock_minutes, self.clock_increment_s));
        self.timeout = None;
        self.resigned = None;
        self.position_changed();
        self.send(EngineCommand::NewGame);
        self.engine_nodes = 0;
        self.engine_lines.clear();
        self.disable_auto_request = false;
    }
}

/// Half the machine's threads, between one and four: enough for a strong
/// opponent without taking the whole machine.
pub fn default_threads() -> usize {
    std::thread::available_parallelism().map_or(1, |n| (n.get() / 2).clamp(1, 4))
}
