use std::str::FromStr;
use chess::{ChessMove, Color as ChessColor,
            File, Piece as ChessPiece, Rank, Square as ChessSquare};
use chess_core::{ChessEngine, GameState, Color, notation, GameHistory};
use chess_engine::{EngineCommand, EngineResponse, StockfishEngine};
use eframe::egui::{self, Response, Ui, Vec2, Pos2, Rect, Color32, CornerRadius};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

pub struct ChessApp {
    engine: ChessEngine,
    game_history: GameHistory,
    selected_square: Option<ChessSquare>,
    legal_moves_for_selected: Vec<ChessMove>,
    board_flip: bool,
    move_history: Vec<String>,
    last_move: Option<(ChessSquare, ChessSquare)>,

    // Board appearance settings
    light_square_color: Color32,
    dark_square_color: Color32,
    selected_square_color: Color32,
    legal_move_color: Color32,
    last_move_color: Color32,

    // Drag and drop state
    dragging_piece: Option<(ChessSquare, ChessPiece, ChessColor)>,
    drag_pos: Option<Pos2>,

    // Stockfish engine integration
    play_vs_computer: bool,
    computer_color: ChessColor,
    stockfish_tx: Option<Sender<EngineCommand>>,
    stockfish_rx: Option<Receiver<EngineResponse>>,
    engine_thinking: bool,
    engine_evaluation: Option<f32>,
    engine_depth: u32,
    engine_nodes: u64,
    engine_best_move: Option<String>,
    engine_pv: Vec<String>,
}

impl ChessApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create channels for engine communication
        let (engine_tx, engine_rx) = channel();
        let (ui_tx, ui_rx) = channel();

        // Spawn engine thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Initialise Stockfish
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

                // Engine event loop
                while let Ok(cmd) = engine_rx.recv() {
                    match cmd {
                        EngineCommand::GetBestMove(fen) => {
                            // Set position
                            let position_cmd = format!("fen {}", fen);
                            if stockfish.set_position(&position_cmd).await.is_err() {
                                continue;
                            }

                            // Start search (depth 15, about 1-2 seconds)
                            if stockfish.go(Some(15), None).await.is_err() {
                                continue;
                            }

                            // Stream responses to UI
                            while let Some(resp) = stockfish.recv_response().await {
                                // Send to UI thread
                                if ui_tx.send(resp.clone()).is_err() {
                                    break;
                                }

                                // Stop when we get bestmove
                                if matches!(resp, EngineResponse::BestMove { .. }) {
                                    break;
                                }
                            }
                        }
                        EngineCommand::Quit => break,
                        _ => {}
                    }
                }

                // Cleanup
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

            // Classic chess colors
            light_square_color: Color32::from_rgb(238, 238, 210),
            dark_square_color: Color32::from_rgb(118, 150, 86),
            selected_square_color: Color32::from_rgba_premultiplied(255, 255, 0, 100),
            legal_move_color: Color32::from_rgba_premultiplied(0, 255, 0, 50),
            last_move_color: Color32::from_rgba_premultiplied(255, 200, 0, 60),

            dragging_piece: None,
            drag_pos: None,

            // Engine state
            play_vs_computer: false,
            computer_color: ChessColor::Black,
            stockfish_tx: Some(engine_tx),
            stockfish_rx: Some(ui_rx),
            engine_thinking: false,
            engine_evaluation: None,
            engine_depth: 0,
            engine_nodes: 0,
            engine_best_move: None,
            engine_pv: Vec::new(),
        }
    }

    fn request_engine_move(&mut self) {
        if !self.play_vs_computer || self.engine_thinking {
            return;
        }

        // Check if it's computer's turn
        let current_turn = if self.engine.side_to_move() == Color::White {
            ChessColor::White
        } else {
            ChessColor::Black
        };

        if current_turn != self.computer_color {
            return;
        }

        // Request move
        if let Some(tx) = &self.stockfish_tx {
            let fen = self.game_history.current_board().to_string();
            let _ = tx.send(EngineCommand::GetBestMove(fen));
            self.engine_thinking = true;
            self.engine_best_move = None;
        }
    }

    fn apply_engine_move(&mut self, move_str: &str) {
        // Parse move in long algebraic notation (e.g. "e2e4")
        if move_str.len() < 4 {
            return;
        }

        let from_str = &move_str[0..2];
        let to_str = &move_str[2..4];

        // Parse squares
        if let (Ok(from), Ok(to)) = (
            ChessSquare::from_str(from_str),
            ChessSquare::from_str(to_str),
        ) {
            // Check for promotion
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

            // Try to make the move
            let full_move_str = format!("{}{}", from_str, to_str);
            if let Some(mv) = notation::parse_algebraic(&full_move_str) {
                if self.engine.make_move(mv).is_ok() {
                    self.game_history.make_move(ChessMove::new(from, to, promotion));
                    self.move_history.push(full_move_str);
                    self.last_move = Some((from, to));
                }
            }
        }
    }

    fn draw_board(&mut self, ui: &mut Ui) -> Response {
        let available_size = ui.available_size();
        let board_size = available_size.x.min(available_size.y - 50.0).max(400.0);
        let square_size = board_size / 8.0;

        let (response, painter) = ui.allocate_painter(
            Vec2::splat(board_size),
            egui::Sense::click_and_drag(),
        );

        let board_rect = response.rect;

        // Draw board squares
        for rank in 0..8 {
            for file in 0..8 {
                let display_rank = if self.board_flip { rank } else { 7 - rank };
                let display_file = if self.board_flip { 7 - file } else { file };

                let square = ChessSquare::make_square(
                    Rank::from_index(display_rank),
                    File::from_index(display_file),
                );

                let square_rect = Rect::from_min_size(
                    board_rect.min + Vec2::new(
                        file as f32 * square_size,
                        rank as f32 * square_size,
                    ),
                    Vec2::splat(square_size),
                );

                // Draw square background
                let is_light = (rank + file) % 2 == 0;
                let mut square_color = if is_light {
                    self.light_square_color
                } else {
                    self.dark_square_color
                };

                // Highlight last move
                if let Some((from, to)) = self.last_move {
                    if square == from || square == to {
                        square_color = Color32::from_rgba_premultiplied(
                            square_color.r(),
                            square_color.g(),
                            square_color.b(),
                            200,
                        );
                        painter.rect_filled(
                            square_rect,
                            CornerRadius::ZERO,
                            self.last_move_color,
                        );
                    }
                }

                painter.rect_filled(square_rect, CornerRadius::ZERO, square_color);

                // Highlight selected square
                if Some(square) == self.selected_square {
                    painter.rect_filled(
                        square_rect,
                        CornerRadius::ZERO,
                        self.selected_square_color,
                    );
                }

                // Highlight legal moves
                if self.legal_moves_for_selected.iter().any(|m| m.get_dest() == square) {
                    let center = square_rect.center();
                    let radius = square_size * 0.15;
                    painter.circle_filled(center, radius, self.legal_move_color);
                }

                // Draw rank and file labels
                if file == 0 {
                    let rank_char = ((display_rank + 1) as u8 + b'0') as char;
                    painter.text(
                        square_rect.left_top() + Vec2::new(2.0, 2.0),
                        egui::Align2::LEFT_TOP,
                        rank_char,
                        egui::FontId::proportional(12.0),
                        if is_light { self.dark_square_color } else { self.light_square_color },
                    );
                }

                if rank == 7 {
                    let file_char = (display_file as u8 + b'a') as char;
                    painter.text(
                        square_rect.right_bottom() - Vec2::new(2.0, 2.0),
                        egui::Align2::RIGHT_BOTTOM,
                        file_char,
                        egui::FontId::proportional(12.0),
                        if is_light { self.dark_square_color } else { self.light_square_color },
                    );
                }

                // Draw pieces (if not being dragged)
                if self.dragging_piece.map_or(true, |(drag_sq, _, _)| drag_sq != square) {
                    // Convert square to our Square type
                    let our_square = chess_core::Square::new(
                        square.get_file().to_index() as u8,
                        square.get_rank().to_index() as u8,
                    ).unwrap();

                    if let Some(piece) = self.engine.piece_at(our_square) {
                        self.draw_piece(&painter, square_rect.center(), piece, square_size * 0.8);
                    }
                }
            }
        }

        // Draw dragging piece
        if let Some((_, piece, color)) = self.dragging_piece {
            if let Some(pos) = self.drag_pos {
                let our_piece = chess_core::Piece {
                    color: if color == ChessColor::White { Color::White } else { Color::Black },
                    piece_type: convert_piece_type(piece),
                };
                self.draw_piece(&painter, pos, our_piece, square_size * 0.8);
            }
        }

        // Handle input (only if it's human's turn
        let is_human_turn = !self.play_vs_computer || {
            let current_turn = if self.engine.side_to_move() == Color::White {
                ChessColor::White
            } else {
                ChessColor::Black
            };
            current_turn != self.computer_color
        };

        if is_human_turn {
            if response.clicked() {
                if let Some(square) = self.get_square_from_pos(
                    response.interact_pointer_pos().unwrap(), board_rect, square_size) {
                        self.handle_square_click(square);
                }
            }

            if response.drag_started() {
                if let Some(square) = self.get_square_from_pos(
                    response
                        .interact_pointer_pos().unwrap(),
                    board_rect,
                    square_size
                ) {
                    let our_square = chess_core::Square::new(
                        square.get_file().to_index() as u8,
                        square.get_rank().to_index() as u8,
                    ).unwrap();

                    if let Some(piece) = self.engine.piece_at(our_square) {
                        if piece.color == self.engine.side_to_move() {
                            // Store as chess crate types for now
                            self.dragging_piece = Some((
                                square,
                                convert_to_chess_piece(piece.piece_type),
                                if piece.color == Color::White { ChessColor::White } else { ChessColor::Black }
                            ));
                            self.selected_square = Some(square);
                            self.update_legal_moves();
                        }
                    }
                }
            }

            if response.dragged() {
                self.drag_pos = response.interact_pointer_pos();
            }

            if response.drag_stopped() {
                if let Some((from_square, _, _)) = self.dragging_piece {
                    if let Some(to_square) = self.get_square_from_pos(
                        response.interact_pointer_pos().unwrap_or(Pos2::ZERO),
                        board_rect,
                        square_size
                    ) {
                        self.try_make_move(from_square, to_square);
                    }
                }
                self.dragging_piece = None;
                self.drag_pos = None;
            }
        }

        response
    }

    fn draw_piece(&self, painter: &egui::Painter, pos: Pos2, piece: chess_core::Piece, size: f32) {
        let piece_char = match piece.piece_type {
            chess_core::PieceType::King => '♚',
            chess_core::PieceType::Queen => '♛',
            chess_core::PieceType::Rook => '♜',
            chess_core::PieceType::Bishop => '♝',
            chess_core::PieceType::Knight => '♞',
            chess_core::PieceType::Pawn => '♟',
        };

        // Draw piece colour
        let text_color = if piece.color == Color::White {
            Color32::from_rgb(255, 255, 255)
        } else {
            Color32::from_rgb(20, 20, 20)
        };

        // Draw piece with outline for better visibility
        let font_size = size * 0.9;
        let font_id = egui::FontId::proportional(font_size);

        // Draw shadow
        painter.text(
            pos + Vec2::new(1.0, 1.0),
            egui::Align2::CENTER_CENTER,
            piece_char,
            font_id.clone(),
            Color32::from_rgba_premultiplied(0, 0, 0, 100),
        );

        // Draw outline
        for dx in [-1.0, 0.0, 1.0] {
            for dy in [-1.0, 0.0, 1.0] {
                if dx != 0.0 || dy != 0.0 {
                    painter.text(
                        pos + Vec2::new(dx * 0.5, dy * 0.5),
                        egui::Align2::CENTER_CENTER,
                        piece_char,
                        font_id.clone(),
                        if piece.color == Color::White {
                            Color32::from_rgb(30, 30, 30)
                        } else {
                            Color32::from_rgb(200, 200, 200)
                        },
                    );
                }
            }
        }

        // Draw piece
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            piece_char,
            font_id,
            text_color,
        );
    }

    fn get_square_from_pos(&self, pos: Pos2, board_rect: Rect, square_size: f32) -> Option<ChessSquare> {
        if !board_rect.contains(pos) {
            return None;
        }

        let relative_pos = pos - board_rect.min;
        let file = (relative_pos.x / square_size) as usize;
        let rank = (relative_pos.y / square_size) as usize;

        if file >= 8 || rank >= 8 {
            return None;
        }

        let display_rank = if self.board_flip { rank } else { 7 - rank };
        let display_file = if self.board_flip { 7 - file } else { file };

        Some(ChessSquare::make_square(
            Rank::from_index(display_rank),
            File::from_index(display_file),
        ))
    }

    fn handle_square_click(&mut self, square: ChessSquare) {
        if let Some(selected) = self.selected_square {
            if selected == square {
                self.selected_square = None;
                self.legal_moves_for_selected.clear();
            } else {
                self.try_make_move(selected, square);
            }
        } else {
            let our_square = chess_core::Square::new(
                square.get_file().to_index() as u8,
                square.get_rank().to_index() as u8,
            ).unwrap();

            if let Some(piece) = self.engine.piece_at(our_square) {
                if piece.color == self.engine.side_to_move() {
                    self.selected_square = Some(square);
                    self.update_legal_moves();
                }
            }
        }
    }

    fn try_make_move(&mut self, from: ChessSquare, to: ChessSquare) {
        let from_str = format!("{}", from);
        let to_str = format!("{}", to);
        let move_str = format!("{}{}", from_str, to_str);

        if let Some(mv) = notation::parse_algebraic(&move_str) {
            if self.engine.make_move(mv).is_ok() {
                self.game_history.make_move(chess::ChessMove::new(from, to, None));  // Also record in game history and convert to chess crate move
                self.move_history.push(move_str);
                self.last_move = Some((from, to));
                self.selected_square = None;
                self.legal_moves_for_selected.clear();

                self.request_engine_move();
                return;
            }
        }

        let our_square = chess_core::Square::new(
            to.get_file().to_index() as u8,
            to.get_rank().to_index() as u8,
        ).unwrap();

        if let Some(piece) = self.engine.piece_at(our_square) {
            if piece.color == self.engine.side_to_move() {
                self.selected_square = Some(to);
                self.update_legal_moves();
                return;
            }
        }

        self.selected_square = None;
        self.legal_moves_for_selected.clear();
    }

    fn update_legal_moves(&mut self) {
        self.legal_moves_for_selected.clear();
        if let Some(square) = self.selected_square {
            let all_moves = self.engine.legal_moves();
            let our_square = chess_core::Square::new(
                square.get_file().to_index() as u8,
                square.get_rank().to_index() as u8,
            ).unwrap();

            for mv in all_moves {
                if mv.from == our_square {
                    let chess_move = ChessMove::new(
                        square,
                        ChessSquare::make_square(
                            Rank::from_index(mv.to.rank() as usize),
                            File::from_index(mv.to.file() as usize),
                        ),
                        mv.promotion.map(convert_to_chess_piece),
                    );
                    self.legal_moves_for_selected.push(chess_move);
                }
            }
        }
    }
}

fn convert_piece_type(piece: ChessPiece) -> chess_core::PieceType {
    match piece {
        ChessPiece::Pawn => chess_core::PieceType::Pawn,
        ChessPiece::Knight => chess_core::PieceType::Knight,
        ChessPiece::Bishop => chess_core::PieceType::Bishop,
        ChessPiece::Rook => chess_core::PieceType::Rook,
        ChessPiece::Queen => chess_core::PieceType::Queen,
        ChessPiece::King => chess_core::PieceType::King,
    }
}

fn convert_to_chess_piece(piece_type: chess_core::PieceType) -> ChessPiece {
    match piece_type {
        chess_core::PieceType::Pawn => ChessPiece::Pawn,
        chess_core::PieceType::Knight => ChessPiece::Knight,
        chess_core::PieceType::Bishop => ChessPiece::Bishop,
        chess_core::PieceType::Rook => ChessPiece::Rook,
        chess_core::PieceType::Queen => ChessPiece::Queen,
        chess_core::PieceType::King => ChessPiece::King,
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll engine responses
        let mut best_move_to_apply: Option<String> = None;

        if let Some(rx) = &self.stockfish_rx {
            loop {
                match rx.try_recv() {
                    Ok(EngineResponse::Info { depth, score, nodes, nps: _, pv}) => {
                        self.engine_depth = depth;
                        self.engine_evaluation = Some(score as f32 / 100.0);
                        self.engine_nodes = nodes;
                        self.engine_pv = pv;
                    }
                    Ok(EngineResponse::BestMove {mv, .. }) => {
                        best_move_to_apply = Some(mv);
                        self.engine_thinking = false;
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        eprintln!("Chess Engine thread disconnected");
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Apply the move after releasing the borrow
        if let Some(mv) = best_move_to_apply {
            self.engine_best_move = Some(mv.clone());
            self.apply_engine_move(&mv);
        }

        // Request repaint while engine is thinking
        if self.engine_thinking {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Game", |ui| {
                    if ui.button("🆕 New Game").clicked() {
                        self.engine = ChessEngine::new();
                        self.game_history = GameHistory::new();
                        self.selected_square = None;
                        self.legal_moves_for_selected.clear();
                        self.move_history.clear();
                        self.last_move = None;
                        self.engine_thinking = false;
                        self.engine_nodes = 0;
                    }

                    ui.separator();

                    if ui.button("🔄 Flip Board").clicked() {
                        self.board_flip = !self.board_flip;
                    }

                    ui.separator();

                    if ui.button("❌ Quit").clicked() {
                        // Send quit to engine thread
                        if let Some(tx) = &self.stockfish_tx {
                            let _ = tx.send(EngineCommand::Quit);
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.label("Board Colors");
                    ui.horizontal(|ui| {
                        ui.label("Light:");
                        ui.color_edit_button_srgba(&mut self.light_square_color);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Dark:");
                        ui.color_edit_button_srgba(&mut self.dark_square_color);
                    });
                });

                ui.separator();

                // Current turn indicator
                let side_to_move = self.engine.side_to_move();
                let turn_text = if side_to_move == Color::White {
                    "⚪ White to move"
                } else {
                    "⚫ Black to move"
                };
                ui.label(egui::RichText::new(turn_text).size(14.0).strong());
            });
        });

        egui::SidePanel::right("right_panel")
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Game Information");
                ui.separator();

                self.draw_game_status(ui);

                ui.separator();

                // Engine controls
                ui.heading("Computer Opponent");
                ui.horizontal(|ui| {
                    ui.label("Play vs Computer");
                   ui.checkbox(&mut self.play_vs_computer, "");
                });

                if self.play_vs_computer {
                    ui.horizontal(|ui| {
                        ui.label("Computer plays:");
                        if ui.radio_value(&mut self.computer_color, ChessColor::White, "White").clicked() {}
                        if ui.radio_value(&mut self.computer_color, ChessColor::Black, "Black").clicked() {}
                    });

                    if self.engine_thinking {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Engine thinking...");
                        });
                    }

                    // Engine analysis display
                    if let Some(eval) = self.engine_evaluation {
                        ui.separator();
                        ui.heading("Engine Analysis");
                        ui.horizontal(|ui| {
                            ui.label("Evaluation:");
                            let eval_text = if eval > 0.0 {
                                format!("+{:.2}", eval)
                            } else {
                                format!("{:.2}", eval)
                            };
                            ui.label(egui::RichText::new(eval_text).strong());
                        });
                        ui.horizontal(|ui| {
                            ui.label("Depth:");
                            ui.label(format!("{}", self.engine_depth));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Nodes:");
                            ui.label(format!("{}", self.engine_nodes));
                        });
                        if !self.engine_pv.is_empty() {
                            ui.label("Principal Variation:");
                            ui.label(self.engine_pv[..self.engine_pv.len().min(5)].join(" "));
                        }
                    }
                }

                ui.separator();

                ui.heading("Move History");
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        if self.move_history.is_empty() {
                            ui.label("No moves yet");
                        } else {
                            for (i, move_pair) in self.move_history.chunks(2).enumerate() {
                                let mut move_text = format!("{}. {}", i + 1, move_pair[0]);
                                if move_pair.len() > 1 {
                                    move_text.push_str(&format!(" {}", move_pair[1]));
                                }
                                ui.label(move_text);
                            }
                        }
                    });

                ui.separator();

                ui.heading("Controls");
                ui.label("• Click to select a piece");
                ui.label("• Click again to move");
                ui.label("• Drag and drop pieces");
                ui.label("• Green dots show legal moves");
            });

        egui::SidePanel::left("left_panel")
            .default_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Position Info");
                ui.separator();

                self.draw_material_count(ui);

                ui.separator();

                if let Some(square) = self.selected_square {
                    ui.heading("Selected Square");
                    ui.label(format!("Square: {}", square));

                    let our_square = chess_core::Square::new(
                        square.get_file().to_index() as u8,
                        square.get_rank().to_index() as u8,
                    ).unwrap();

                    if let Some(piece) = self.engine.piece_at(our_square) {
                        ui.label(format!("Piece: {:?} {:?}", piece.color, piece.piece_type));
                        ui.label(format!("Legal moves: {}", self.legal_moves_for_selected.len()));
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Chess Board");

                self.draw_board(ui);

                ui.horizontal(|ui| {
                    if ui.button("🔄 New Game").clicked() {
                        self.engine = ChessEngine::new();
                        self.game_history = GameHistory::new();
                        self.selected_square = None;
                        self.legal_moves_for_selected.clear();
                        self.move_history.clear();
                        self.last_move = None;
                        self.engine_thinking = false;
                        self.engine_evaluation = None;
                    }

                    if ui.button("🔃 Flip Board").clicked() {
                        self.board_flip = !self.board_flip;
                    }

                    ui.separator();

                    if ui.add_enabled(self.game_history.can_undo(), egui::Button::new("⬅ Undo")).clicked() {
                        if self.game_history.undo() {
                            self.sync_engine_from_history();
                            self.selected_square = None;
                            self.legal_moves_for_selected.clear();
                        }
                    }

                    if ui.add_enabled(self.game_history.can_redo(), egui::Button::new("➡ Redo")).clicked() {
                        if self.game_history.redo() {
                            self.sync_engine_from_history();
                            self.selected_square = None;
                            self.legal_moves_for_selected.clear();
                        }
                    }
                });
            });
        });
    }
}

impl ChessApp {
    fn draw_game_status(&self, ui: &mut Ui) {
        if self.engine.is_checkmate() {
            let winner = if self.engine.side_to_move() == Color::White {
                "Black wins by checkmate!"
            } else {
                "White wins by checkmate!"
            };
            ui.colored_label(Color32::from_rgb(255, 100, 100), winner);
        } else if self.engine.is_stalemate() {
            ui.colored_label(Color32::from_rgb(255, 200, 100), "Stalemate - Draw!");
        } else if self.engine.is_check() {
            ui.colored_label(Color32::from_rgb(255, 150, 50), "Check!");
        } else {
            ui.label("Game in progress");
        }
    }

    fn draw_material_count(&self, ui: &mut Ui) {
        ui.label("Material:");

        let mut white_material = 0;
        let mut black_material = 0;

        for rank in 0..8 {
            for file in 0..8 {
                if let Some(sq) = chess_core::Square::new(file, rank) {
                    if let Some(piece) = self.engine.piece_at(sq) {
                        let value = match piece.piece_type {
                            chess_core::PieceType::Pawn => 1,
                            chess_core::PieceType::Knight | chess_core::PieceType::Bishop => 3,
                            chess_core::PieceType::Rook => 5,
                            chess_core::PieceType::Queen => 9,
                            chess_core::PieceType::King => 0,
                        };

                        if piece.color == Color::White {
                            white_material += value;
                        } else {
                            black_material += value;
                        }
                    }
                }
            }
        }

        ui.horizontal(|ui| {
            ui.label("⚪ White:");
            ui.label(format!("{}", white_material));
        });

        ui.horizontal(|ui| {
            ui.label("⚫ Black:");
            ui.label(format!("{}", black_material));
        });

        let diff = white_material - black_material;
        if diff > 0 {
            ui.colored_label(Color32::from_rgb(200, 200, 200), format!("White +{}", diff));
        } else if diff < 0 {
            ui.colored_label(Color32::from_rgb(100, 100, 100), format!("Black +{}", -diff));
        } else {
            ui.label("Equal material");
        }
    }

    fn sync_engine_from_history(&mut self) {
        let fen = self.game_history.current_board().to_string();
        self.engine = ChessEngine::from_fen(&fen).unwrap_or_else(|_| ChessEngine::new());

        self.move_history.clear();
        for i in 0..self.game_history.move_count() {
            if let Some(mv) = self.game_history.get_move(i) {
                let from = mv.get_source();
                let to = mv.get_dest();
                let move_str = format!("{}{}", from, to);
                self.move_history.push(move_str);
            }
        }
    }
}