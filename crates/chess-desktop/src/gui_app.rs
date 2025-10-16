//! Chess GUI application module.
//!
//! This is the main entry point that re-exports the modularized application.

use chess_desktop::ChessApp;
use chess_desktop::ui::theme::Theme;

#[cfg(test)]
mod tests {
    use super::*;
    use chess::Board;
    use std::str::FromStr;
    use chess::Piece as ChessPiece;
    use chess_core::GameHistory;
    use chess_desktop::ui::theme::ThemeVariant::ClassicMonochrome;

    #[test]
    fn test_parse_uci_move_basic() {
        let board = Board::default();
        let dummy_app = create_test_app();

        let chess_move = dummy_app.parse_uci_move("e2e4", &board);
        assert!(chess_move.is_some());

        let mv = chess_move.unwrap();
        assert_eq!(mv.get_source().to_string(), "e2");
        assert_eq!(mv.get_dest().to_string(), "e4");
    }

    #[test]
    fn test_parse_uci_move_promotion() {
        let board = Board::from_str(
            "4k3/P7/8/8/8/8/8/4K3 w - - 0 1"
        ).unwrap();

        let dummy_app = create_test_app();

        let chess_move = dummy_app.parse_uci_move("a7a8q", &board);
        assert!(chess_move.is_some());
        assert_eq!(chess_move.unwrap().get_promotion(), Some(ChessPiece::Queen));
    }

    #[test]
    fn test_parse_uci_move_invalid() {
        let board = Board::default();
        let dummy_app = create_test_app();

        assert!(dummy_app.parse_uci_move("e2e5", &board).is_none());
        assert!(dummy_app.parse_uci_move("e2", &board).is_none());
        assert!(dummy_app.parse_uci_move("xyz", &board).is_none());
    }

    #[test]
    fn test_format_pv_san_basic() {
        let dummy_app = create_test_app();
        let pv = vec![
            "e2e4".to_string(),
            "e7e5".to_string(),
            "g1f3".to_string(),
        ];

        let formatted = dummy_app.format_pv_san(&pv);
        assert_eq!(formatted, vec!["e4", "e5", "Nf3"]);
    }

    #[test]
    fn test_format_pv_san_with_capture() {
        let mut dummy_app = create_test_app();

        let board = Board::from_str(
            "r1bqkbnr/ppp2ppp/2n5/3pp3/3PP3/5N2/PPP2PPP/RNBQKB1R w KQkq d6 0 4"
        ).unwrap();
        dummy_app.game_history = GameHistory::from_board(board);

        let pv = vec!["e4d5".to_string()];
        let formatted = dummy_app.format_pv_san(&pv);
        assert_eq!(formatted, vec!["exd5"]);
    }

    fn create_test_app() -> ChessApp {
        use chess::Color as ChessColor;
        use eframe::egui::Color32;
        use chess_desktop::app::{CapturedPiecesStyle, EngineMode};

        ChessApp {
            engine: chess_core::ChessEngine::new(),
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
            stockfish_tx: None,
            stockfish_rx: None,
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
            theme: Theme::classic_monochrome(),
            theme_variant: ClassicMonochrome,
        }
    }
}