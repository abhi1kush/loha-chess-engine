//! Loha, a Rust port of the original C chess engine.
//!
//! Search depends on traits for evaluation, the opening book, and protocol
//! output. The board stores a position. FEN, diagrams, and move text live in
//! their own modules.

pub mod adjudicate;
pub mod attack;
pub mod board;
pub mod console;
pub mod control;
pub mod defs;
pub mod evaluate;
pub mod fen;
pub mod hash;
pub mod input;
pub mod listener;
pub mod makemove;
pub mod movegen;
pub mod notation;
pub mod perft;
pub mod polybook;
pub mod polykeys;
pub mod ports;
pub mod search;
pub mod tables;
pub mod uci;
pub mod validate;
pub mod view;
pub mod xboard;

pub use board::Board;
pub use defs::{AUTHOR, NAME, START_FEN};
pub use perft::{perft, perft_divide, run_suite};
pub use ports::SearchInfo;
