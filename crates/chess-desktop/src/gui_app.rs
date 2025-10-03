use chess::{ChessMove, Color as ChessColor,
            File, Piece as ChessPiece, Rank, Square as ChessSquare};
use chess_core::{ChessEngine, GameState, Color, notation, GameHistory};
use eframe::egui::{self, Response, Ui, Vec2, Pos2, Rect, Color32,
                   CornerRadius};

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
}

impl ChessApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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

        // Handle input
        if response.clicked() {
            if let Some(square) = self.get_square_from_pos(response.interact_pointer_pos().unwrap(), board_rect, square_size) {
                self.handle_square_click(square);
            }
        }

        if response.drag_started() {
            if let Some(square) = self.get_square_from_pos(response.interact_pointer_pos().unwrap(), board_rect, square_size) {
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
                // Deselect
                self.selected_square = None;
                self.legal_moves_for_selected.clear();
            } else {
                // Try to make a move
                self.try_make_move(selected, square);
            }
        } else {
            // Check if there's a piece on this square
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
        // Convert to our notation
        let from_str = format!("{}", from);
        let to_str = format!("{}", to);
        let move_str = format!("{}{}", from_str, to_str);

        // Try to parse and make the move
        if let Some(mv) = notation::parse_algebraic(&move_str) {
            if self.engine.make_move(mv).is_ok() {
                self.game_history.make_move(chess::ChessMove::new(from, to, None));  // Also record in game history and convert to chess crate move
                self.move_history.push(move_str);
                self.last_move = Some((from, to));
                self.selected_square = None;
                self.legal_moves_for_selected.clear();
                return;
            }
        }

        // If move failed, check if clicking on own piece
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
            // Get legal moves from engine
            let all_moves = self.engine.legal_moves();

            // Convert our square
            let our_square = chess_core::Square::new(
                square.get_file().to_index() as u8,
                square.get_rank().to_index() as u8,
            ).unwrap();

            // Filter moves from this square
            for mv in all_moves {
                if mv.from == our_square {
                    // Create a chess crate move for display
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

// Helper functions for piece type conversion
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

// eframe::App implementation
impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with menu
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
                    }

                    ui.separator();

                    if ui.button("🔄 Flip Board").clicked() {
                        self.board_flip = !self.board_flip;
                    }

                    ui.separator();

                    if ui.button("❌ Quit").clicked() {
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

        // Right panel with game info
        egui::SidePanel::right("right_panel")
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Game Information");
                ui.separator();

                // Game status
                self.draw_game_status(ui);

                ui.separator();

                // Move history
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

                // Controls
                ui.heading("Controls");
                ui.label("• Click to select a piece");
                ui.label("• Click again to move");
                ui.label("• Drag and drop pieces");
                ui.label("• Green dots show legal moves");
            });

        // Left panel with current position info
        egui::SidePanel::left("left_panel")
            .default_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Position Info");
                ui.separator();

                // Material count
                self.draw_material_count(ui);

                ui.separator();

                // Selected square info
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

        // Central panel with chess board
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Chess Board");

                // Draw the chess board
                self.draw_board(ui);

                // Quick action buttons below the board
                ui.horizontal(|ui| {
                    // New game button
                    if ui.button("🔄 New Game").clicked() {
                        self.engine = ChessEngine::new();
                        self.game_history = GameHistory::new();
                        self.selected_square = None;
                        self.legal_moves_for_selected.clear();
                        self.move_history.clear();
                        self.last_move = None;
                    }

                    // Flip board button
                    if ui.button("🔃 Flip Board").clicked() {
                        self.board_flip = !self.board_flip;
                    }

                    ui.separator();

                    // Undo button
                    if ui.add_enabled(self.game_history.can_undo(), egui::Button::new("⬅ Undo")).clicked() {
                        if self.game_history.undo() {
                            self.sync_engine_from_history();
                            self.selected_square = None;
                            self.legal_moves_for_selected.clear();
                        }
                    }

                    // Redo button
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

// Helper methods for UI components
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

        // Count material (simple point system)
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

        let diff = white_material as i32 - black_material as i32;
        if diff > 0 {
            ui.colored_label(Color32::from_rgb(200, 200, 200), format!("White +{}", diff));
        } else if diff < 0 {
            ui.colored_label(Color32::from_rgb(100, 100, 100), format!("Black +{}", -diff));
        } else {
            ui.label("Equal material");
        }
    }

    fn sync_engine_from_history(&mut self) {
        // Reconstruct engine state from current history position
        let fen = self.game_history.current_board().to_string();
        self.engine = ChessEngine::from_fen(&fen).unwrap_or_else(|_| ChessEngine::new());

        // Update move history display
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