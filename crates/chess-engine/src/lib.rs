//! Async UCI client for a local Stockfish process.

pub mod stockfish;

pub use stockfish::{EngineResponse, Score, SearchLimit, StockfishEngine};
