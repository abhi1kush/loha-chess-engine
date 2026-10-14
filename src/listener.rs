//! Search output for each protocol. Adding a protocol does not change search.

use crate::board::Board;
use crate::defs::NOMOVE;
use crate::input::write_line;
use crate::makemove::make_move;
use crate::notation::move_string;
use crate::ports::SearchListener;

pub struct UciListener;

impl SearchListener for UciListener {
    fn on_iteration(
        &mut self,
        board: &Board,
        nodes: u64,
        score: i32,
        depth: i32,
        elapsed_ms: u64,
        pv_len: i32,
    ) {
        let mut line =
            format!("info score cp {score} depth {depth} nodes {nodes} time {elapsed_ms} pv");
        append_pv(&mut line, board, pv_len);
        write_line(&line);
    }

    fn on_best_move(&mut self, _board: &mut Board, mov: u32) {
        write_line(&format!("bestmove {}", move_string(mov)));
    }
}

pub struct XboardListener;

impl SearchListener for XboardListener {
    fn on_iteration(
        &mut self,
        board: &Board,
        nodes: u64,
        score: i32,
        depth: i32,
        elapsed_ms: u64,
        pv_len: i32,
    ) {
        let mut line = format!("{depth} {score} {} {nodes}", elapsed_ms / 10);
        append_pv(&mut line, board, pv_len);
        write_line(&line);
    }

    fn on_best_move(&mut self, board: &mut Board, mov: u32) {
        write_line(&format!("move {}", move_string(mov)));
        play(board, mov);
    }
}

pub struct ConsoleListener {
    pub post: bool,
}

impl SearchListener for ConsoleListener {
    fn on_iteration(
        &mut self,
        board: &Board,
        nodes: u64,
        score: i32,
        depth: i32,
        elapsed_ms: u64,
        pv_len: i32,
    ) {
        if !self.post {
            return;
        }
        let mut line = format!("score {score} depth {depth} nodes {nodes} time {elapsed_ms}(ms)");
        append_pv(&mut line, board, pv_len);
        write_line(&line);
    }

    fn on_best_move(&mut self, board: &mut Board, mov: u32) {
        write_line(&format!(
            "\n\n***!! engine makes move {} !!*** \n",
            move_string(mov)
        ));
        play(board, mov);
        print!("{board}");
    }
}

pub struct SilentListener;

impl SearchListener for SilentListener {
    fn on_iteration(
        &mut self,
        _board: &Board,
        _nodes: u64,
        _score: i32,
        _depth: i32,
        _elapsed_ms: u64,
        _pv_len: i32,
    ) {
    }

    fn on_best_move(&mut self, _board: &mut Board, _mov: u32) {}
}

fn append_pv(line: &mut String, board: &Board, pv_len: i32) {
    for i in 0..pv_len {
        line.push(' ');
        line.push_str(&move_string(board.pv_array[i as usize]));
    }
}

fn play(board: &mut Board, mov: u32) {
    if mov != NOMOVE {
        make_move(board, mov);
        board.ply = 0;
    }
}
