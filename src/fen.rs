//! Forsyth-Edwards notation. The board stores a position; this module reads one.

use crate::board::Board;
use crate::defs::*;
use crate::tables::sq120;

pub fn parse(board: &mut Board, fen: &str) -> bool {
    board.reset();
    let bytes = fen.as_bytes();
    let mut i = 0usize;
    let mut rank = RANK_8;
    let mut file = FILE_A;

    while rank >= RANK_1 && i < bytes.len() {
        let c = bytes[i];
        let piece;
        let mut count = 1i32;
        match c {
            b'p' => piece = BP,
            b'r' => piece = BR,
            b'n' => piece = BN,
            b'b' => piece = BB,
            b'k' => piece = BK,
            b'q' => piece = BQ,
            b'P' => piece = WP,
            b'R' => piece = WR,
            b'N' => piece = WN,
            b'B' => piece = WB,
            b'K' => piece = WK,
            b'Q' => piece = WQ,
            b'1'..=b'8' => {
                piece = EMPTY;
                count = (c - b'0') as i32;
            }
            b'/' | b' ' => {
                rank -= 1;
                file = FILE_A;
                i += 1;
                continue;
            }
            _ => {
                println!("FEN error");
                return false;
            }
        }
        for _ in 0..count {
            let sq = sq120(rank * 8 + file);
            if piece != EMPTY {
                board.pieces[sq as usize] = piece;
            }
            file += 1;
        }
        i += 1;
    }

    if i >= bytes.len() {
        println!("FEN error");
        return false;
    }
    board.side = if bytes[i] == b'w' { WHITE } else { BLACK };
    i += 1;
    if i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }

    for _ in 0..4 {
        if i >= bytes.len() || bytes[i] == b' ' {
            break;
        }
        match bytes[i] {
            b'K' => board.castle_perm |= WKCA,
            b'Q' => board.castle_perm |= WQCA,
            b'k' => board.castle_perm |= BKCA,
            b'q' => board.castle_perm |= BQCA,
            _ => {}
        }
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }

    if i < bytes.len() && bytes[i] != b'-' {
        if i + 1 >= bytes.len() {
            println!("FEN error");
            return false;
        }
        let ep_file = (bytes[i] - b'a') as i32;
        let ep_rank = (bytes[i + 1] - b'1') as i32;
        board.en_passant = fr2sq(ep_file, ep_rank);
        i += 2;
    } else if i < bytes.len() && bytes[i] == b'-' {
        i += 1;
    }

    if i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    if i < bytes.len() && bytes[i].is_ascii_digit() {
        let mut halfmove = 0i32;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            halfmove = halfmove * 10 + (bytes[i] - b'0') as i32;
            i += 1;
        }
        board.fifty_move = halfmove;
    }

    board.pos_key = board.generate_pos_key();
    board.update_lists_material();
    true
}
