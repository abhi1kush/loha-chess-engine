//! Square-attack detection on the 120-square mailbox.

use crate::board::Board;
use crate::defs::*;

const KNIGHT_DIR: [i32; 8] = [-8, -19, -21, -12, 8, 19, 21, 12];
const ROOK_DIR: [i32; 4] = [-1, -10, 1, 10];
const BISHOP_DIR: [i32; 4] = [-9, -11, 11, 9];
const KING_DIR: [i32; 8] = [-1, -10, 1, 10, -9, -11, 11, 9];

pub fn sq_attacked(board: &Board, sq: i32, side: usize) -> bool {
    if side == WHITE {
        if board.pieces[(sq - 11) as usize] == WP || board.pieces[(sq - 9) as usize] == WP {
            return true;
        }
    } else if board.pieces[(sq + 11) as usize] == BP || board.pieces[(sq + 9) as usize] == BP {
        return true;
    }

    for dir in KNIGHT_DIR {
        let piece = board.pieces[(sq + dir) as usize];
        if piece != OFFBOARD as u8
            && PIECE_KNIGHT[piece as usize]
            && PIECE_COL[piece as usize] == side
        {
            return true;
        }
    }

    for dir in ROOK_DIR {
        let mut t_sq = sq + dir;
        let mut piece = board.pieces[t_sq as usize];
        while piece != OFFBOARD as u8 {
            if piece != EMPTY {
                if PIECE_RQ[piece as usize] && PIECE_COL[piece as usize] == side {
                    return true;
                }
                break;
            }
            t_sq += dir;
            piece = board.pieces[t_sq as usize];
        }
    }

    for dir in BISHOP_DIR {
        let mut t_sq = sq + dir;
        let mut piece = board.pieces[t_sq as usize];
        while piece != OFFBOARD as u8 {
            if piece != EMPTY {
                if PIECE_BQ[piece as usize] && PIECE_COL[piece as usize] == side {
                    return true;
                }
                break;
            }
            t_sq += dir;
            piece = board.pieces[t_sq as usize];
        }
    }

    for dir in KING_DIR {
        let piece = board.pieces[(sq + dir) as usize];
        if piece != OFFBOARD as u8
            && PIECE_KING[piece as usize]
            && PIECE_COL[piece as usize] == side
        {
            return true;
        }
    }

    false
}
