//! Chess board rendering and interaction.
//!
//! Handles board drawing, piece rendering, drag-and-drop, and square selection.

use chess::{ChessMove, Color as ChessColor, File, Rank, Square as ChessSquare};
use chess_core::{Color, GameState};
use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Response, Ui, Vec2};

use crate::app::state::ChessApp;
use crate::utils::conversions::{convert_piece_type, convert_to_chess_piece};

impl ChessApp {
    /// Draw the chess board with pieces and interactions
    pub fn draw_board(&mut self, ui: &mut Ui) -> Response {
        let available_size = ui.available_size();
        let board_size = available_size.x.min(available_size.y);
        let square_size = board_size / 8.0;

        let (response, painter) =
            ui.allocate_painter(Vec2::splat(board_size), egui::Sense::click_and_drag());

        let board_rect = response.rect;

        // Draw all squares and pieces
        for rank in 0..8 {
            for file in 0..8 {
                let display_rank = if self.board_flip { rank } else { 7 - rank };
                let display_file = if self.board_flip { 7 - file } else { file };

                let square = ChessSquare::make_square(
                    Rank::from_index(display_rank),
                    File::from_index(display_file),
                );

                let square_rect = Rect::from_min_size(
                    board_rect.min
                        + Vec2::new(file as f32 * square_size, rank as f32 * square_size),
                    Vec2::splat(square_size),
                );

                self.draw_square(
                    square,
                    square_rect,
                    rank,
                    file,
                    display_rank,
                    display_file,
                    square_size,
                    &painter,
                );
                self.draw_square_labels(
                    rank,
                    file,
                    display_rank,
                    display_file,
                    square_rect,
                    &painter,
                );
                self.draw_piece_on_square(square, square_rect, square_size, &painter);
            }
        }

        // Draw dragging piece on top
        self.draw_dragging_piece(square_size, &painter);

        // Handle user interactions
        self.handle_board_interactions(&response, board_rect, square_size);

        response
    }

    /// Draw a single square with highlighting
    #[allow(clippy::too_many_arguments)]
    fn draw_square(
        &self,
        square: ChessSquare,
        square_rect: Rect,
        rank: usize,
        file: usize,
        _display_rank: usize,
        _display_file: usize,
        square_size: f32,
        painter: &egui::Painter,
    ) {
        let is_light = (rank + file).is_multiple_of(2);
        let mut square_color = if is_light {
            self.light_square_color
        } else {
            self.dark_square_color
        };

        // Highlight last move
        if let Some((from, to)) = self.last_move
            && (square == from || square == to)
        {
            square_color = Color32::from_rgba_premultiplied(
                square_color.r(),
                square_color.g(),
                square_color.b(),
                200,
            );
            painter.rect_filled(square_rect, CornerRadius::ZERO, self.last_move_color);
        }

        painter.rect_filled(square_rect, CornerRadius::ZERO, square_color);

        // Highlight selected square
        if Some(square) == self.selected_square {
            painter.rect_filled(square_rect, CornerRadius::ZERO, self.selected_square_color);
        }

        // Highlight legal moves
        if self
            .legal_moves_for_selected
            .iter()
            .any(|m| m.get_dest() == square)
        {
            let center = square_rect.center();
            let radius = square_size * 0.15;
            painter.circle_filled(center, radius, self.legal_move_color);
        }
    }

    /// Draw rank and file labels on board edges
    fn draw_square_labels(
        &self,
        rank: usize,
        file: usize,
        display_rank: usize,
        display_file: usize,
        square_rect: Rect,
        painter: &egui::Painter,
    ) {
        let is_light = (rank + file).is_multiple_of(2);

        if file == 0 {
            let rank_char = ((display_rank + 1) as u8 + b'0') as char;
            painter.text(
                square_rect.left_top() + Vec2::new(2.0, 2.0),
                egui::Align2::LEFT_TOP,
                rank_char,
                egui::FontId::proportional(12.0),
                if is_light {
                    self.dark_square_color
                } else {
                    self.light_square_color
                },
            );
        }

        if rank == 7 {
            let file_char = (display_file as u8 + b'a') as char;
            painter.text(
                square_rect.right_bottom() - Vec2::new(2.0, 2.0),
                egui::Align2::RIGHT_BOTTOM,
                file_char,
                egui::FontId::proportional(12.0),
                if is_light {
                    self.dark_square_color
                } else {
                    self.light_square_color
                },
            );
        }
    }

    /// Draw piece on a square (if not being dragged)
    fn draw_piece_on_square(
        &self,
        square: ChessSquare,
        square_rect: Rect,
        square_size: f32,
        painter: &egui::Painter,
    ) {
        if self
            .dragging_piece
            .is_none_or(|(drag_sq, _, _)| drag_sq != square)
        {
            let our_square = chess_core::Square::new(
                square.get_file().to_index() as u8,
                square.get_rank().to_index() as u8,
            )
            .unwrap();

            if let Some(piece) = self.engine.piece_at(our_square) {
                draw_piece(painter, square_rect.center(), piece, square_size * 0.8);
            }
        }
    }

    /// Draw piece being dragged by user
    fn draw_dragging_piece(&self, square_size: f32, painter: &egui::Painter) {
        if let Some((_, piece, color)) = self.dragging_piece
            && let Some(pos) = self.drag_pos
        {
            let our_piece = chess_core::Piece {
                color: if color == ChessColor::White {
                    Color::White
                } else {
                    Color::Black
                },
                piece_type: convert_piece_type(piece),
            };
            draw_piece(painter, pos, our_piece, square_size * 0.8);
        }
    }

    /// Handle all board interactions (clicks, drags)
    fn handle_board_interactions(
        &mut self,
        response: &Response,
        board_rect: Rect,
        square_size: f32,
    ) {
        let is_human_turn = !self.play_vs_computer || {
            let current_turn = if self.engine.side_to_move() == Color::White {
                ChessColor::White
            } else {
                ChessColor::Black
            };
            current_turn != self.computer_color
        };

        if !is_human_turn {
            return;
        }

        // Handle click
        if response.clicked()
            && let Some(square) = crate::utils::coords::get_square_from_pos(
                response.interact_pointer_pos().unwrap(),
                board_rect,
                square_size,
                self.board_flip,
            )
        {
            self.handle_square_click(square);
        }

        // Handle drag start
        if response.drag_started()
            && let Some(square) = crate::utils::coords::get_square_from_pos(
                response.interact_pointer_pos().unwrap(),
                board_rect,
                square_size,
                self.board_flip,
            )
        {
            self.start_drag(square);
        }

        // Handle dragging
        if response.dragged() {
            self.drag_pos = response.interact_pointer_pos();
        }

        // Handle drag end
        if response.drag_stopped() {
            if let Some((from_square, _, _)) = self.dragging_piece
                && let Some(to_square) = crate::utils::coords::get_square_from_pos(
                    response.interact_pointer_pos().unwrap_or(Pos2::ZERO),
                    board_rect,
                    square_size,
                    self.board_flip,
                )
            {
                self.try_make_move(from_square, to_square);
            }
            self.dragging_piece = None;
            self.drag_pos = None;
        }
    }

    /// Handle square click for piece selection
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
            )
            .unwrap();

            if let Some(piece) = self.engine.piece_at(our_square)
                && piece.color == self.engine.side_to_move()
            {
                self.selected_square = Some(square);
                self.update_legal_moves();
            }
        }
    }

    /// Start dragging a piece
    fn start_drag(&mut self, square: ChessSquare) {
        let our_square = chess_core::Square::new(
            square.get_file().to_index() as u8,
            square.get_rank().to_index() as u8,
        )
        .unwrap();

        if let Some(piece) = self.engine.piece_at(our_square)
            && piece.color == self.engine.side_to_move()
        {
            self.dragging_piece = Some((
                square,
                convert_to_chess_piece(piece.piece_type),
                if piece.color == Color::White {
                    ChessColor::White
                } else {
                    ChessColor::Black
                },
            ));
            self.selected_square = Some(square);
            self.update_legal_moves();
        }
    }

    /// Attempt to make a move from one square to another
    fn try_make_move(&mut self, from: ChessSquare, to: ChessSquare) {
        let from_str = format!("{}", from);
        let to_str = format!("{}", to);
        let move_str = format!("{}{}", from_str, to_str);

        if let Some(mv) = chess_core::notation::parse_algebraic(&move_str)
            && self.engine.make_move(mv).is_ok()
        {
            let mut legal_moves = chess::MoveGen::new_legal(self.game_history.current_board());
            let chess_move = legal_moves
                .find(|m| m.get_source() == from && m.get_dest() == to)
                .unwrap_or(ChessMove::new(from, to, None));

            let san = chess_core::notation::format_move_san(
                &chess_move,
                self.game_history.current_board(),
            );

            self.game_history.make_move(chess_move);
            self.move_history.push(san);
            self.last_move = Some((from, to));
            self.viewing_move_index = None;
            self.selected_square = None;
            self.legal_moves_for_selected.clear();
            self.disable_auto_request = false;

            return;
        }

        // If move failed, try to select the destination square
        let our_square = chess_core::Square::new(
            to.get_file().to_index() as u8,
            to.get_rank().to_index() as u8,
        )
        .unwrap();

        if let Some(piece) = self.engine.piece_at(our_square)
            && piece.color == self.engine.side_to_move()
        {
            self.selected_square = Some(to);
            self.update_legal_moves();
            return;
        }

        self.selected_square = None;
        self.legal_moves_for_selected.clear();
    }

    /// Update list of legal moves for selected square
    fn update_legal_moves(&mut self) {
        self.legal_moves_for_selected.clear();
        if let Some(square) = self.selected_square {
            let all_moves = self.engine.legal_moves();
            let our_square = chess_core::Square::new(
                square.get_file().to_index() as u8,
                square.get_rank().to_index() as u8,
            )
            .unwrap();

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

/// Draw a chess piece at given position
fn draw_piece(painter: &egui::Painter, pos: Pos2, piece: chess_core::Piece, size: f32) {
    let piece_char = match piece.piece_type {
        chess_core::PieceType::King => '♚',
        chess_core::PieceType::Queen => '♛',
        chess_core::PieceType::Rook => '♜',
        chess_core::PieceType::Bishop => '♝',
        chess_core::PieceType::Knight => '♞',
        chess_core::PieceType::Pawn => '♟',
    };

    let text_color = if piece.color == chess_core::Color::White {
        Color32::from_rgb(255, 255, 255)
    } else {
        Color32::from_rgb(20, 20, 20)
    };

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

    // Draw outline for contrast
    for dx in [-1.0, 0.0, 1.0] {
        for dy in [-1.0, 0.0, 1.0] {
            if dx != 0.0 || dy != 0.0 {
                painter.text(
                    pos + Vec2::new(dx * 0.5, dy * 0.5),
                    egui::Align2::CENTER_CENTER,
                    piece_char,
                    font_id.clone(),
                    if piece.color == chess_core::Color::White {
                        Color32::from_rgb(30, 30, 30)
                    } else {
                        Color32::from_rgb(200, 200, 200)
                    },
                );
            }
        }
    }

    // Draw main piece
    painter.text(
        pos,
        egui::Align2::CENTER_CENTER,
        piece_char,
        font_id,
        text_color,
    );
}
