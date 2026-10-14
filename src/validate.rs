//! Position consistency checks used while debugging make and unmake.

use crate::board::Board;
use crate::defs::*;
use crate::tables::{rank_of, sq120};

pub fn consistent(board: &Board) -> bool {
    let mut piece_num = [0i32; 13];
    let mut big = [0i32; 2];
    let mut major = [0i32; 2];
    let mut minor = [0i32; 2];
    let mut material = [0i32; 2];

    for piece in WP..=BK {
        for n in 0..board.piece_num[piece as usize] {
            let sq = board.piece_list[piece as usize][n as usize];
            if board.pieces[sq as usize] != piece {
                return false;
            }
        }
    }
    for s64 in 0..64 {
        let sq = sq120(s64 as i32);
        let piece = board.pieces[sq as usize];
        piece_num[piece as usize] += 1;
        let color = PIECE_COL[piece as usize];
        if piece == EMPTY {
            continue;
        }
        if PIECE_BIG[piece as usize] {
            big[color] += 1;
        }
        if PIECE_MIN[piece as usize] {
            minor[color] += 1;
        }
        if PIECE_MAJ[piece as usize] {
            major[color] += 1;
        }
        material[color] += PIECE_VAL[piece as usize];
    }
    for piece in WP..=BK {
        if piece_num[piece as usize] != board.piece_num[piece as usize] {
            return false;
        }
    }
    if board.pawns[WHITE].count_ones() != board.piece_num[WP as usize] as u32 {
        return false;
    }
    if board.pawns[BLACK].count_ones() != board.piece_num[BP as usize] as u32 {
        return false;
    }
    if board.pawns[BOTH].count_ones()
        != (board.piece_num[WP as usize] + board.piece_num[BP as usize]) as u32
    {
        return false;
    }
    let mut white_pawns = board.pawns[WHITE];
    while white_pawns != 0 {
        let s64 = white_pawns.trailing_zeros() as i32;
        white_pawns &= white_pawns - 1;
        if board.pieces[sq120(s64) as usize] != WP {
            return false;
        }
    }
    let mut black_pawns = board.pawns[BLACK];
    while black_pawns != 0 {
        let s64 = black_pawns.trailing_zeros() as i32;
        black_pawns &= black_pawns - 1;
        if board.pieces[sq120(s64) as usize] != BP {
            return false;
        }
    }
    if material != board.material
        || minor != board.minor_piece
        || major != board.major_piece
        || big != board.big_piece
    {
        return false;
    }
    if board.side != WHITE && board.side != BLACK {
        return false;
    }
    if board.generate_pos_key() != board.pos_key {
        return false;
    }
    if board.en_passant != NO_SQ {
        let rank = rank_of(board.en_passant);
        let ok = (rank == RANK_6 && board.side == WHITE) || (rank == RANK_3 && board.side == BLACK);
        if !ok {
            return false;
        }
    }
    if board.pieces[board.king_sq[WHITE] as usize] != WK {
        return false;
    }
    if board.pieces[board.king_sq[BLACK] as usize] != BK {
        return false;
    }
    true
}
