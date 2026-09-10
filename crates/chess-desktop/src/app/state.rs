//! Application state management.
//!
//! Contains the ChessApp struct and all game state.

use chess::{Board, ChessMove, Color as ChessColor, Piece as ChessPiece, Square as ChessSquare};
use chess_core::GameHistory;
use chess_engine::Score;
use eframe::egui::Pos2;
use std::sync::mpsc::{Receiver, channel};
use tokio::sync::mpsc::UnboundedSender;

use crate::ui::theme::{Theme, ThemeVariant};

use super::engine_comm::EngineMode;
use super::engine_link::{self, EngineCommand, EngineEvent, EngineStatus};

/// Style for displaying captured pieces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapturedPiecesStyle {
    /// Show only the advantage (Lichess style)
    Lichess,
    /// Show all captured pieces for both sides (Chess.com style)
    ChessCom,
}

/// Main application state
pub struct ChessApp {
    // Game state
    pub game_history: GameHistory,
    pub selected_square: Option<ChessSquare>,
    pub legal_moves_for_selected: Vec<ChessMove>,
    pub last_move: Option<(ChessSquare, ChessSquare)>,
    /// A pawn move waiting for the player to choose the promotion piece.
    pub pending_promotion: Option<(ChessSquare, ChessSquare)>,
    /// Text of the "Set up position" window while it is open.
    pub fen_input: Option<String>,

    // UI state
    pub board_flip: bool,
    pub dragging_piece: Option<(ChessSquare, ChessPiece, ChessColor)>,
    pub drag_pos: Option<Pos2>,
    pub show_eval_bar: bool,
    pub captured_display_style: CapturedPiecesStyle,

    // Engine state
    pub play_vs_computer: bool,
    pub computer_color: ChessColor,
    pub engine_tx: Option<UnboundedSender<EngineCommand>>,
    pub engine_rx: Option<Receiver<EngineEvent>>,
    pub engine_status: EngineStatus,
    /// Id of the latest search; replies to any other id are stale.
    pub search_id: u64,
    pub engine_thinking: bool,
    pub engine_evaluation: Option<Score>,
    pub engine_depth_current: u32,
    pub engine_nodes: u64,
    pub engine_pv: Vec<String>,
    pub engine_depth: u32,
    pub engine_movetime: Option<u64>,
    pub engine_mode: EngineMode,
    pub engine_skill_level: i32,

    /// Set while the user browses the history, so the engine does not reply
    /// to a position that is not the live one.
    pub disable_auto_request: bool,

    // UI theme
    pub theme: Theme,
    pub theme_variant: ThemeVariant,
}

impl ChessApp {
    /// Create the app and start the Stockfish thread.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (engine_tx, commands) = tokio::sync::mpsc::unbounded_channel();
        let (events, engine_rx) = channel();
        engine_link::spawn(commands, events, cc.egui_ctx.clone());

        let app = Self {
            engine_tx: Some(engine_tx),
            engine_rx: Some(engine_rx),
            ..Self::headless()
        };
        app.theme.apply(&cc.egui_ctx);
        app
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
            last_move: None,
            pending_promotion: None,
            fen_input: None,
            disable_auto_request: false,
            dragging_piece: None,
            drag_pos: None,
            play_vs_computer: false,
            computer_color: ChessColor::Black,
            engine_tx: None,
            engine_rx: None,
            engine_status: EngineStatus::Starting,
            search_id: 0,
            engine_thinking: false,
            engine_evaluation: None,
            engine_depth_current: 0,
            engine_nodes: 0,
            engine_pv: Vec::new(),
            engine_depth: 20,
            engine_movetime: Some(1000),
            engine_mode: EngineMode::Depth,
            engine_skill_level: 20,
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

    /// The piece on `square`, with its colour.
    pub fn piece_at(&self, square: ChessSquare) -> Option<(ChessPiece, ChessColor)> {
        let board = self.board();
        Some((board.piece_on(square)?, board.color_on(square)?))
    }

    /// Reset the game to initial position
    pub fn new_game(&mut self) {
        self.start_game(GameHistory::new());
    }

    /// Replace the game with `history` and tell the engine to forget the old one.
    pub fn start_game(&mut self, history: GameHistory) {
        self.game_history = history;
        self.position_changed();
        self.send(EngineCommand::NewGame);
        self.last_move = None;
        self.engine_nodes = 0;
        self.engine_depth_current = 0;
        self.engine_pv.clear();
        self.engine_evaluation = None;
        self.disable_auto_request = false;
    }
}
