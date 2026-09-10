//! Application state management.
//!
//! Contains the ChessApp struct and all game state.

use chess::{ChessMove, Color as ChessColor, Piece as ChessPiece, Square as ChessSquare};
use chess_core::{ChessEngine, GameHistory};
use chess_engine::Score;
use eframe::egui::{Color32, Pos2};
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
    pub engine: ChessEngine,
    pub game_history: GameHistory,
    pub selected_square: Option<ChessSquare>,
    pub legal_moves_for_selected: Vec<ChessMove>,
    pub last_move: Option<(ChessSquare, ChessSquare)>,
    /// A pawn move waiting for the player to choose the promotion piece.
    pub pending_promotion: Option<(ChessSquare, ChessSquare)>,

    // UI state
    pub board_flip: bool,
    pub light_square_color: Color32,
    pub dark_square_color: Color32,
    pub selected_square_color: Color32,
    pub legal_move_color: Color32,
    pub last_move_color: Color32,
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

        Self {
            engine_tx: Some(engine_tx),
            engine_rx: Some(engine_rx),
            ..Self::headless()
        }
    }

    /// The app with no engine thread, for tests and for `new` to build on.
    pub fn headless() -> Self {
        Self {
            engine: ChessEngine::new(),
            game_history: GameHistory::new(),
            selected_square: None,
            legal_moves_for_selected: Vec::new(),
            board_flip: false,
            last_move: None,
            pending_promotion: None,
            disable_auto_request: false,
            light_square_color: Color32::from_rgb(238, 238, 210),
            dark_square_color: Color32::from_rgb(118, 150, 86),
            selected_square_color: Color32::from_rgba_premultiplied(255, 255, 0, 100),
            legal_move_color: Color32::from_rgba_premultiplied(0, 255, 0, 50),
            last_move_color: Color32::from_rgba_premultiplied(255, 200, 0, 60),
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

    /// Reset the game to initial position
    pub fn new_game(&mut self) {
        self.game_history = GameHistory::new();
        self.sync_engine();
        self.send(EngineCommand::NewGame);
        self.last_move = None;
        self.engine_nodes = 0;
        self.engine_depth_current = 0;
        self.engine_pv.clear();
        self.engine_evaluation = None;
        self.disable_auto_request = false;
    }
}
