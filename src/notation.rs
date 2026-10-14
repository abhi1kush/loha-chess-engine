//! Algebraic move text. Parsing and printing stay out of the board and the protocols.

use crate::board::Board;
use crate::defs::*;
use crate::movegen::generate_all_moves;
use crate::tables::{file_of, rank_of};

pub fn move_string(mov: u32) -> String {
    if mov == NOMOVE {
        return "0000".to_string();
    }
    let from = from_sq(mov);
    let to = to_sq(mov);
    if file_of(from) == OFFBOARD || file_of(to) == OFFBOARD {
        return "0000".to_string();
    }
    let mut text = format!(
        "{}{}{}{}",
        (b'a' + file_of(from) as u8) as char,
        (b'1' + rank_of(from) as u8) as char,
        (b'a' + file_of(to) as u8) as char,
        (b'1' + rank_of(to) as u8) as char,
    );
    let prom = promoted(mov);
    if prom != EMPTY {
        let ch = if PIECE_KNIGHT[prom as usize] {
            'n'
        } else if PIECE_RQ[prom as usize] && !PIECE_BQ[prom as usize] {
            'r'
        } else if !PIECE_RQ[prom as usize] && PIECE_BQ[prom as usize] {
            'b'
        } else {
            'q'
        };
        text.push(ch);
    }
    text
}

pub fn parse_move(text: &str, board: &Board) -> u32 {
    let bytes = text.as_bytes();
    if bytes.len() < 4 {
        return NOMOVE;
    }
    if !(b'1'..=b'8').contains(&bytes[1]) || !(b'1'..=b'8').contains(&bytes[3]) {
        return NOMOVE;
    }
    if !(b'a'..=b'h').contains(&bytes[0]) || !(b'a'..=b'h').contains(&bytes[2]) {
        return NOMOVE;
    }
    let from = fr2sq((bytes[0] - b'a') as i32, (bytes[1] - b'1') as i32);
    let to = fr2sq((bytes[2] - b'a') as i32, (bytes[3] - b'1') as i32);
    let mut list = MoveList::default();
    generate_all_moves(board, &mut list);
    for n in 0..list.count {
        let mov = list.moves[n].mov;
        if from_sq(mov) != from || to_sq(mov) != to {
            continue;
        }
        let prom = promoted(mov);
        if prom != EMPTY {
            let flag = bytes.get(4).copied().unwrap_or(0);
            if PIECE_RQ[prom as usize] && !PIECE_BQ[prom as usize] && flag == b'r' {
                return mov;
            } else if !PIECE_RQ[prom as usize] && PIECE_BQ[prom as usize] && flag == b'b' {
                return mov;
            } else if PIECE_RQ[prom as usize] && PIECE_BQ[prom as usize] && flag == b'q' {
                return mov;
            } else if PIECE_KNIGHT[prom as usize] && flag == b'n' {
                return mov;
            }
            continue;
        }
        return mov;
    }
    NOMOVE
}

pub fn print_move_list(list: &MoveList) {
    println!("MoveList:{}", list.count);
    for (i, entry) in list.moves.iter().take(list.count).enumerate() {
        println!(
            "Move:{} > {} (score:{})",
            i + 1,
            move_string(entry.mov),
            entry.score
        );
    }
    println!("MoveList Total {} Moves:\n", list.count);
}
