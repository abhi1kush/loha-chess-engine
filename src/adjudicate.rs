//! Game-ending claims: mate, stalemate, repetition, and the fifty-move rule.

use crate::attack::sq_attacked;
use crate::board::Board;
use crate::defs::*;
use crate::input::write_line;
use crate::makemove::{make_move, take_move};
use crate::movegen::generate_all_moves;

fn three_fold(board: &Board) -> i32 {
    let mut repeats = 0;
    for i in 0..board.history_ply {
        if board.history[i as usize].pos_key == board.pos_key {
            repeats += 1;
        }
    }
    repeats
}

fn draw_material(board: &Board) -> bool {
    if board.piece_num[WP as usize] != 0 || board.piece_num[BP as usize] != 0 {
        return false;
    }
    if board.piece_num[WQ as usize] != 0
        || board.piece_num[BQ as usize] != 0
        || board.piece_num[WR as usize] != 0
        || board.piece_num[BR as usize] != 0
    {
        return false;
    }
    if board.piece_num[WB as usize] > 1 || board.piece_num[BB as usize] > 1 {
        return false;
    }
    if board.piece_num[WN as usize] > 1 || board.piece_num[BN as usize] > 1 {
        return false;
    }
    if board.piece_num[WN as usize] != 0 && board.piece_num[WB as usize] != 0 {
        return false;
    }
    if board.piece_num[BN as usize] != 0 && board.piece_num[BB as usize] != 0 {
        return false;
    }
    true
}

pub fn game_over(board: &mut Board) -> bool {
    if board.fifty_move > 100 {
        write_line("1/2-1/2 {fifty move rule (claimed by loha)}");
        return true;
    }
    if three_fold(board) >= 2 {
        write_line("1/2-1/2 {3-fold repetition (claimed by loha)}");
        return true;
    }
    if draw_material(board) {
        write_line("1/2-1/2 {insufficient material (claimed by loha)}");
        return true;
    }

    let mut list = MoveList::default();
    generate_all_moves(board, &mut list);
    for n in 0..list.count {
        if !make_move(board, list.moves[n].mov) {
            continue;
        }
        take_move(board);
        return false;
    }
    let in_check = sq_attacked(board, board.king_sq[board.side], board.side ^ 1);
    if in_check {
        if board.side == WHITE {
            write_line("0-1 {black mates (claimed by loha)}");
        } else {
            write_line("1-0 {white mates (claimed by loha)}");
        }
    } else {
        write_line("1/2-1/2 {stalemate (claimed by loha)}");
    }
    true
}
