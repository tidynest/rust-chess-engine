//! Chess Desktop Library
//!
//! Core application logic and UI components for the chess desktop application.

// Declare all modules at crate root
pub mod app;
pub mod ui;
pub mod utils;

// Re-export the main application struct for convenience
pub use app::ChessApp;