//! Chess board rendering and interaction.
//!
//! Handles board drawing, piece rendering, drag-and-drop, and square selection.

use cozy_chess::{
    Color as ChessColor, File, Move, Piece as ChessPiece, Rank, Square as ChessSquare,
};
use eframe::egui::{self, Context, CornerRadius, Rect, Response, Ui, Vec2};
use std::time::Duration;

use crate::app::state::ChessApp;
use crate::ui::pieces;
use chess_core::{moves, notation};

/// How long a piece takes to slide to its square.
const SLIDE_TIME: Duration = Duration::from_millis(150);

impl ChessApp {
    /// Draw the chess board with pieces and interactions
    pub fn draw_board(&mut self, ui: &mut Ui) -> Response {
        let available_size = ui.available_size();
        let board_size = available_size.x.min(available_size.y);
        let square_size = board_size / 8.0;

        let (response, painter) =
            ui.allocate_painter(Vec2::splat(board_size), egui::Sense::click_and_drag());

        let board_rect = response.rect;

        let board_flip = self.board_flip;
        let square_at = |pos| {
            crate::utils::coords::get_square_from_pos(pos, board_rect, square_size, board_flip)
        };
        // The square a dragged piece would land on.
        let hover = self.dragging_piece.and(self.drag_pos).and_then(square_at);
        let checked_king = {
            let board = self.board();
            (!board.checkers().is_empty()).then(|| board.king(board.side_to_move()))
        };
        // A piece still on its way is drawn between squares, not on one.
        let sliding = self.animation.and_then(|(from, to, started)| {
            let t = started.elapsed().as_secs_f32() / SLIDE_TIME.as_secs_f32();
            (t < 1.0).then_some((from, to, t))
        });
        if sliding.is_some() {
            ui.ctx().request_repaint();
        }
        let hidden = self
            .dragging_piece
            .map(|(square, _, _)| square)
            .or(sliding.map(|(_, to, _)| to));

        // Draw all squares and pieces
        for rank in 0..8 {
            for file in 0..8 {
                let display_rank = if self.board_flip { rank } else { 7 - rank };
                let display_file = if self.board_flip { 7 - file } else { file };

                let square = ChessSquare::new(File::index(display_file), Rank::index(display_rank));

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
                    square_size,
                    &painter,
                    hover,
                    checked_king,
                );
                self.draw_square_labels(
                    rank,
                    file,
                    display_rank,
                    display_file,
                    square_rect,
                    &painter,
                );
                if Some(square) != hidden
                    && let Some((piece, color)) = self.piece_at(square)
                {
                    pieces::draw(&painter, square_rect.center(), square_size, piece, color);
                }
            }
        }

        if let Some((from, to, t)) = sliding
            && let Some((piece, color)) = self.piece_at(to)
        {
            let from = self.square_center(from, board_rect, square_size);
            let to = self.square_center(to, board_rect, square_size);
            // Ease out: quick off the square, settling on arrival.
            let eased = 1.0 - (1.0 - t) * (1.0 - t);
            pieces::draw(&painter, from.lerp(to, eased), square_size, piece, color);
        }

        self.draw_engine_arrows(board_rect, square_size, &painter);

        // Draw dragging piece on top
        self.draw_dragging_piece(square_size, &painter);

        // Handle user interactions
        self.handle_board_interactions(&response, board_rect, square_size);

        response
    }

    /// Draw a single square, then its highlights on top.
    #[allow(clippy::too_many_arguments)]
    fn draw_square(
        &self,
        square: ChessSquare,
        square_rect: Rect,
        rank: usize,
        file: usize,
        square_size: f32,
        painter: &egui::Painter,
        hover: Option<ChessSquare>,
        checked_king: Option<ChessSquare>,
    ) {
        let theme = &self.theme;
        let is_light = (rank + file).is_multiple_of(2);
        let square_color = if is_light {
            theme.board_light
        } else {
            theme.board_dark
        };
        painter.rect_filled(square_rect, CornerRadius::ZERO, square_color);

        // Highlight last move
        if let Some((from, to)) = self.last_move()
            && (square == from || square == to)
        {
            painter.rect_filled(square_rect, CornerRadius::ZERO, theme.last_move);
        }

        // Highlight selected square
        if Some(square) == self.selected_square {
            painter.rect_filled(square_rect, CornerRadius::ZERO, theme.selected);
        }

        // A king in check, and the square a dragged piece hovers over
        if Some(square) == checked_king {
            painter.rect_filled(
                square_rect,
                CornerRadius::ZERO,
                theme.check.gamma_multiply(0.5),
            );
        }
        if Some(square) == hover {
            painter.rect_filled(square_rect, CornerRadius::ZERO, theme.hover);
        }

        // Highlight legal moves
        if self
            .legal_moves_for_selected
            .iter()
            .any(|&m| moves::destination(self.board(), m) == square)
        {
            let center = square_rect.center();
            let radius = square_size * 0.15;
            painter.circle_filled(center, radius, theme.legal_move);
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
        let theme = &self.theme;
        let is_light = (rank + file).is_multiple_of(2);
        let label_color = if is_light {
            theme.board_dark
        } else {
            theme.board_light
        };
        let font = egui::FontId::proportional(theme.font_size_xs);

        if file == 0 {
            let rank_char = ((display_rank + 1) as u8 + b'0') as char;
            painter.text(
                square_rect.left_top() + Vec2::new(2.0, 2.0),
                egui::Align2::LEFT_TOP,
                rank_char,
                font.clone(),
                label_color,
            );
        }

        if rank == 7 {
            let file_char = (display_file as u8 + b'a') as char;
            painter.text(
                square_rect.right_bottom() - Vec2::new(2.0, 2.0),
                egui::Align2::RIGHT_BOTTOM,
                file_char,
                font,
                label_color,
            );
        }
    }

    /// Screen centre of `square`, honouring the board flip.
    fn square_center(&self, square: ChessSquare, board_rect: Rect, square_size: f32) -> egui::Pos2 {
        let (rank, file) = ((square.rank() as usize), (square.file() as usize));
        let row = if self.board_flip { rank } else { 7 - rank };
        let col = if self.board_flip { 7 - file } else { file };
        board_rect.min + Vec2::new(col as f32 + 0.5, row as f32 + 0.5) * square_size
    }

    /// In analysis mode, an arrow for the first move of each engine line,
    /// the best line boldest and on top.
    fn draw_engine_arrows(&self, board_rect: Rect, square_size: f32, painter: &egui::Painter) {
        if !self.analysis {
            return;
        }
        for (index, line) in self.engine_lines.iter().enumerate().rev() {
            let Some(mv) = line
                .pv
                .first()
                .and_then(|uci| notation::parse_uci(self.board(), uci))
            else {
                continue;
            };
            let from = self.square_center(mv.from, board_rect, square_size);
            let to = self.square_center(
                moves::destination(self.board(), mv),
                board_rect,
                square_size,
            );
            let alpha = if index == 0 { 0.7 } else { 0.35 };
            let stroke =
                egui::Stroke::new(square_size * 0.12, self.theme.accent.gamma_multiply(alpha));
            painter.arrow(from, to - from, stroke);
        }
    }

    /// Draw piece being dragged by user
    fn draw_dragging_piece(&self, square_size: f32, painter: &egui::Painter) {
        if let Some((_, piece, color)) = self.dragging_piece
            && let Some(pos) = self.drag_pos
        {
            pieces::draw(painter, pos, square_size, piece, color);
        }
    }

    /// Handle all board interactions (clicks, drags)
    fn handle_board_interactions(
        &mut self,
        response: &Response,
        board_rect: Rect,
        square_size: f32,
    ) {
        // While the engine is not searching, the human may move either side:
        // after browsing the history that is how play resumes.
        if self.waiting_for_engine_move() || self.pending_promotion.is_some() || self.is_game_over()
        {
            return;
        }

        let board_flip = self.board_flip;
        let square_at = |pos| {
            crate::utils::coords::get_square_from_pos(pos, board_rect, square_size, board_flip)
        };

        // Handle click
        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
            && let Some(square) = square_at(pos)
        {
            self.handle_square_click(square);
        }

        // Handle drag start
        if response.drag_started()
            && let Some(pos) = response.interact_pointer_pos()
            && let Some(square) = square_at(pos)
        {
            self.start_drag(square);
        }

        // Handle dragging
        if response.dragged() {
            self.drag_pos = response.interact_pointer_pos();
        }

        // Handle drag end; a drop outside the board cancels the drag
        if response.drag_stopped() {
            if let Some((from_square, _, _)) = self.dragging_piece
                && let Some(pos) = response.interact_pointer_pos()
                && let Some(to_square) = square_at(pos)
            {
                self.try_make_move(from_square, to_square);
                // The piece was dropped where it lands; nothing to slide.
                self.animation = None;
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
        } else if self.own_piece_at(square).is_some() {
            self.selected_square = Some(square);
            self.update_legal_moves();
        }
    }

    /// Start dragging a piece
    fn start_drag(&mut self, square: ChessSquare) {
        if let Some((piece, color)) = self.own_piece_at(square) {
            self.dragging_piece = Some((square, piece, color));
            self.selected_square = Some(square);
            self.update_legal_moves();
        }
    }

    /// The piece on `square` if it belongs to the side to move.
    fn own_piece_at(&self, square: ChessSquare) -> Option<(ChessPiece, ChessColor)> {
        self.piece_at(square)
            .filter(|&(_, color)| color == self.board().side_to_move())
    }

    /// Play the legal move between two squares, ask for the piece if it is a
    /// promotion, or move the selection when there is no such move.
    fn try_make_move(&mut self, from: ChessSquare, to: ChessSquare) {
        let board = self.game_history.current_board();
        let legal = moves::legal_moves(board)
            .into_iter()
            .find(|&m| m.from == from && moves::destination(board, m) == to);

        match legal {
            Some(mv) if mv.promotion.is_none() => return self.play_move(mv),
            Some(_) => {
                self.pending_promotion = Some((from, to));
                return;
            }
            None => {}
        }

        // If move failed, try to select the destination square
        self.selected_square = self.own_piece_at(to).map(|_| to);
        self.update_legal_moves();
    }

    /// Modal choice of the promotion piece for the pending pawn move.
    pub fn draw_promotion_picker(&mut self, ctx: &Context) {
        let Some((from, to)) = self.pending_promotion else {
            return;
        };

        let color = self.board().side_to_move();
        let mut open = true;
        let mut choice = None;
        egui::Window::new("Promote to")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for piece in [
                        ChessPiece::Queen,
                        ChessPiece::Rook,
                        ChessPiece::Bishop,
                        ChessPiece::Knight,
                    ] {
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::splat(64.0), egui::Sense::click());
                        if response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                self.theme.border_radius,
                                self.theme.hover,
                            );
                        }
                        pieces::draw(ui.painter(), rect.center(), 64.0, piece, color);
                        if response.clicked() {
                            choice = Some(piece);
                        }
                    }
                });
            });

        if let Some(piece) = choice {
            self.play_move(Move {
                from,
                to,
                promotion: Some(piece),
            });
        } else if !open {
            self.position_changed();
        }
    }

    /// Update list of legal moves for selected square
    fn update_legal_moves(&mut self) {
        self.legal_moves_for_selected.clear();
        if let Some(square) = self.selected_square {
            self.legal_moves_for_selected.extend(
                moves::legal_moves(self.board())
                    .into_iter()
                    .filter(|mv| mv.from == square),
            );
        }
    }
}
