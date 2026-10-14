//! Boundaries the rest of the engine programs against.
//!
//! Search depends on these traits, not on UCI, XBoard, the Polyglot file, or
//! the classic evaluation tables. A new book, evaluator, or protocol implements
//! the same small interface and replaces the old one without changing search.

use crate::board::Board;

/// Limits, clock, and the stop flag for one search.
#[derive(Clone, Debug)]
pub struct SearchInfo {
    pub start_time: u64,
    pub stop_time: u64,
    pub depth: i32,
    pub time_set: bool,
    pub nodes: u64,
    pub quit: bool,
    pub stopped: bool,
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            start_time: 0,
            stop_time: 0,
            depth: crate::defs::MAX_DEPTH,
            time_set: false,
            nodes: 0,
            quit: false,
            stopped: false,
        }
    }
}

/// Position evaluation. Search never names a concrete scorer.
pub trait Evaluator {
    fn evaluate(&self, board: &Board) -> i32;
}

/// Opening book. `NOMOVE` means the book has nothing to play.
pub trait OpeningBook {
    fn suggest(&mut self, board: &Board) -> u32;
}

/// Turns a book on or off. Search does not use this.
pub trait BookSwitch {
    fn set_enabled(&mut self, enabled: bool);
}

/// One search report. Protocols format it; search does not.
pub trait SearchListener {
    fn on_iteration(
        &mut self,
        board: &Board,
        nodes: u64,
        score: i32,
        depth: i32,
        elapsed_ms: u64,
        pv_len: i32,
    );

    fn on_best_move(&mut self, board: &mut Board, mov: u32);
}

/// Clock and GUI interrupts. Tests supply a control that never stops the search.
pub trait SearchControl {
    fn now_ms(&self) -> u64;
    fn poll(&mut self, info: &mut SearchInfo);
}
