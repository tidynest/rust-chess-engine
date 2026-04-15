//! Application state management.
//!
//! Contains the ChessApp struct and all game state.

use chess::{ChessMove, Color as ChessColor, Piece as ChessPiece, Square as ChessSquare};
use chess_core::{ChessEngine, GameHistory};
use chess_engine::{EngineCommand, EngineResponse, StockfishEngine};
use eframe::egui::{Color32, Pos2};
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::ui::theme::{Theme, ThemeVariant};

use super::engine_comm::EngineMode;

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
    pub move_history: Vec<String>,
    pub last_move: Option<(ChessSquare, ChessSquare)>,
    pub viewing_move_index: Option<usize>,
    pub skip_history_rebuild: bool,

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
    pub stockfish_tx: Option<Sender<EngineCommand>>,
    pub stockfish_rx: Option<Receiver<EngineResponse>>,
    pub engine_thinking: bool,
    pub engine_evaluation: Option<f32>,
    pub engine_depth_current: u32,
    pub engine_nodes: u64,
    pub engine_best_move: Option<String>,
    pub engine_pv: Vec<String>,
    pub engine_depth: u32,
    pub engine_movetime: Option<u64>,
    pub engine_mode: EngineMode,
    pub engine_skill_level: i32,

    // Internal state
    pub last_move_count_check: usize,
    pub loop_protection_counter: u8,
    pub disable_auto_request: bool,
    pub _show_engine_settings: bool,

    // UI theme
    pub theme: Theme,
    pub theme_variant: ThemeVariant,
}

impl ChessApp {
    /// Create a new ChessApp instance
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (engine_tx, engine_rx) = channel();
        let (ui_tx, ui_rx) = channel();

        // Spawn Stockfish engine thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut stockfish = match StockfishEngine::new("stockfish").await {
                    Ok(engine) => engine,
                    Err(e) => {
                        eprintln!("Failed to start Stockfish: {}", e);
                        return;
                    }
                };

                if let Err(e) = stockfish.initialise().await {
                    eprintln!("Failed to initialise Stockfish: {}", e);
                    return;
                }

                while let Ok(cmd) = engine_rx.recv() {
                    match cmd {
                        EngineCommand::GetBestMove {
                            fen,
                            depth,
                            movetime,
                            skill_level,
                        } => {
                            if skill_level < 20 {
                                let _ = stockfish
                                    .send_command(&format!(
                                        "setoption name Skill Level value {}",
                                        skill_level
                                    ))
                                    .await;
                                let _ = stockfish.wait_ready().await;
                            } else {
                                let _ = stockfish
                                    .send_command("setoption name Skill Level value 20")
                                    .await;
                                let _ = stockfish.wait_ready().await;
                            }

                            let position_cmd = format!("fen {}", fen);
                            if stockfish.set_position(&position_cmd).await.is_err() {
                                continue;
                            }

                            if stockfish.go(depth, movetime).await.is_err() {
                                continue;
                            }

                            while let Some(resp) = stockfish.recv_response().await {
                                if ui_tx.send(resp.clone()).is_err() {
                                    break;
                                }

                                if matches!(resp, EngineResponse::BestMove { .. }) {
                                    break;
                                }
                            }
                        }
                        EngineCommand::Quit => break,
                    }
                }

                let _ = stockfish.quit().await;
            });
        });

        Self {
            engine: ChessEngine::new(),
            game_history: GameHistory::new(),
            selected_square: None,
            legal_moves_for_selected: Vec::new(),
            board_flip: false,
            move_history: Vec::new(),
            last_move: None,
            last_move_count_check: 0,
            loop_protection_counter: 0,
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
            stockfish_tx: Some(engine_tx),
            stockfish_rx: Some(ui_rx),
            engine_thinking: false,
            engine_evaluation: None,
            engine_depth_current: 0,
            engine_nodes: 0,
            engine_best_move: None,
            engine_pv: Vec::new(),
            viewing_move_index: None,
            skip_history_rebuild: false,
            engine_depth: 20,
            engine_movetime: Some(1000),
            engine_mode: EngineMode::Depth,
            engine_skill_level: 20,
            _show_engine_settings: false,
            show_eval_bar: true,
            captured_display_style: CapturedPiecesStyle::Lichess,
            theme: Theme::default(),
            theme_variant: ThemeVariant::ClassicMonochrome,
        }
    }

    /// Reset the game to initial position
    pub fn new_game(&mut self) {
        self.engine = ChessEngine::new();
        self.game_history = GameHistory::new();
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
        self.move_history.clear();
        self.last_move = None;
        self.engine_thinking = false;
        self.engine_nodes = 0;
        self.disable_auto_request = false;
        self.engine_evaluation = None;
    }
}
