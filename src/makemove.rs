//! Make and unmake moves, including castling, en passant, and promotions.

use crate::attack::sq_attacked;
use crate::board::Board;
use crate::defs::*;
use crate::tables::{clear_bit, set_bit, sq64};

const CASTLE_PERM: [u8; 120] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 0
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 10
    15, 13, 15, 15, 15, 12, 15, 15, 14, 15, // 20
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 30
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 40
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 50
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 60
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 70
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 80
    15, 7, 15, 15, 15, 3, 15, 15, 11, 15, // 90
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 100
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, // 110
];

pub fn make_null_move(board: &mut Board) {
    board.ply += 1;
    board.history[board.history_ply as usize].pos_key = board.pos_key;
    if board.en_passant != NO_SQ {
        board.hash_ep();
    }
    let slot = &mut board.history[board.history_ply as usize];
    slot.mov = NOMOVE;
    slot.fifty_move = board.fifty_move;
    slot.en_passant = board.en_passant;
    slot.castle = board.castle_perm;
    board.en_passant = NO_SQ;
    board.side ^= 1;
    board.history_ply += 1;
    board.hash_side();
}

pub fn take_null_move(board: &mut Board) {
    board.history_ply -= 1;
    board.ply -= 1;
    if board.en_passant != NO_SQ {
        board.hash_ep();
    }
    let undo = board.history[board.history_ply as usize];
    board.castle_perm = undo.castle;
    board.fifty_move = undo.fifty_move;
    board.en_passant = undo.en_passant;
    if board.en_passant != NO_SQ {
        board.hash_ep();
    }
    board.side ^= 1;
    board.hash_side();
}

fn clear_piece(board: &mut Board, sq: i32) {
    let piece = board.pieces[sq as usize];
    let color = PIECE_COL[piece as usize];
    board.hash_piece(piece, sq);
    board.pieces[sq as usize] = EMPTY;
    board.material[color] -= PIECE_VAL[piece as usize];
    if PIECE_BIG[piece as usize] {
        board.big_piece[color] -= 1;
        if PIECE_MAJ[piece as usize] {
            board.major_piece[color] -= 1;
        } else {
            board.minor_piece[color] -= 1;
        }
    } else {
        let s64 = sq64(sq);
        clear_bit(&mut board.pawns[color], s64);
        clear_bit(&mut board.pawns[BOTH], s64);
    }
    let mut found = None;
    for i in 0..board.piece_num[piece as usize] {
        if board.piece_list[piece as usize][i as usize] == sq {
            found = Some(i);
            break;
        }
    }
    let found = found.expect("piece list missing square");
    board.piece_num[piece as usize] -= 1;
    let last = board.piece_num[piece as usize];
    board.piece_list[piece as usize][found as usize] =
        board.piece_list[piece as usize][last as usize];
}

fn add_piece(board: &mut Board, sq: i32, piece: u8) {
    let color = PIECE_COL[piece as usize];
    board.hash_piece(piece, sq);
    board.pieces[sq as usize] = piece;
    if PIECE_BIG[piece as usize] {
        board.big_piece[color] += 1;
        if PIECE_MAJ[piece as usize] {
            board.major_piece[color] += 1;
        } else {
            board.minor_piece[color] += 1;
        }
    } else {
        let s64 = sq64(sq);
        set_bit(&mut board.pawns[color], s64);
        set_bit(&mut board.pawns[BOTH], s64);
    }
    board.material[color] += PIECE_VAL[piece as usize];
    let index = board.piece_num[piece as usize] as usize;
    board.piece_list[piece as usize][index] = sq;
    board.piece_num[piece as usize] += 1;
}

fn move_piece(board: &mut Board, from: i32, to: i32) {
    let piece = board.pieces[from as usize];
    let color = PIECE_COL[piece as usize];
    board.hash_piece(piece, from);
    board.pieces[from as usize] = EMPTY;
    board.hash_piece(piece, to);
    board.pieces[to as usize] = piece;
    if !PIECE_BIG[piece as usize] {
        let from64 = sq64(from);
        let to64 = sq64(to);
        clear_bit(&mut board.pawns[color], from64);
        clear_bit(&mut board.pawns[BOTH], from64);
        set_bit(&mut board.pawns[color], to64);
        set_bit(&mut board.pawns[BOTH], to64);
    }
    for i in 0..board.piece_num[piece as usize] {
        if board.piece_list[piece as usize][i as usize] == from {
            board.piece_list[piece as usize][i as usize] = to;
            break;
        }
    }
}

pub fn make_move(board: &mut Board, mov: u32) -> bool {
    let from = from_sq(mov);
    let to = to_sq(mov);
    let side = board.side;

    board.history[board.history_ply as usize].pos_key = board.pos_key;

    if (mov & MFLAG_EP) != 0 {
        if side == WHITE {
            clear_piece(board, to - 10);
        } else {
            clear_piece(board, to + 10);
        }
    } else if (mov & MFLAG_CA) != 0 {
        match to {
            C1 => move_piece(board, A1, D1),
            C8 => move_piece(board, A8, D8),
            G1 => move_piece(board, H1, F1),
            G8 => move_piece(board, H8, F8),
            _ => panic!("castle destination {to}"),
        }
    }

    if board.en_passant != NO_SQ {
        board.hash_ep();
    }
    board.hash_castle();

    let slot = &mut board.history[board.history_ply as usize];
    slot.mov = mov;
    slot.fifty_move = board.fifty_move;
    slot.en_passant = board.en_passant;
    slot.castle = board.castle_perm;

    board.castle_perm &= CASTLE_PERM[from as usize];
    board.castle_perm &= CASTLE_PERM[to as usize];
    board.en_passant = NO_SQ;
    board.hash_castle();

    let cap = captured(mov);
    board.fifty_move += 1;
    if cap != EMPTY {
        clear_piece(board, to);
        board.fifty_move = 0;
    }

    board.history_ply += 1;
    board.ply += 1;

    if PIECE_PAWN[board.pieces[from as usize] as usize] {
        board.fifty_move = 0;
        if (mov & MFLAG_PS) != 0 {
            if side == WHITE {
                board.en_passant = from + 10;
            } else {
                board.en_passant = from - 10;
            }
            board.hash_ep();
        }
    }

    move_piece(board, from, to);

    let prom = promoted(mov);
    if prom != EMPTY {
        clear_piece(board, to);
        add_piece(board, to, prom);
    }
    if PIECE_KING[board.pieces[to as usize] as usize] {
        board.king_sq[board.side] = to;
    }

    board.side ^= 1;
    board.hash_side();

    if sq_attacked(board, board.king_sq[side], board.side) {
        take_move(board);
        return false;
    }
    debug_assert!(board.check_board());
    true
}

pub fn take_move(board: &mut Board) {
    board.history_ply -= 1;
    board.ply -= 1;

    let mov = board.history[board.history_ply as usize].mov;
    let from = from_sq(mov);
    let to = to_sq(mov);

    if board.en_passant != NO_SQ {
        board.hash_ep();
    }
    board.hash_castle();

    let undo = board.history[board.history_ply as usize];
    board.castle_perm = undo.castle;
    board.fifty_move = undo.fifty_move;
    board.en_passant = undo.en_passant;

    if board.en_passant != NO_SQ {
        board.hash_ep();
    }
    board.hash_castle();

    board.side ^= 1;
    board.hash_side();

    if (mov & MFLAG_EP) != 0 {
        if board.side == WHITE {
            add_piece(board, to - 10, BP);
        } else {
            add_piece(board, to + 10, WP);
        }
    } else if (mov & MFLAG_CA) != 0 {
        match to {
            C1 => move_piece(board, D1, A1),
            C8 => move_piece(board, D8, A8),
            G1 => move_piece(board, F1, H1),
            G8 => move_piece(board, F8, H8),
            _ => panic!("castle destination {to}"),
        }
    }

    move_piece(board, to, from);
    if PIECE_KING[board.pieces[from as usize] as usize] {
        board.king_sq[board.side] = from;
    }

    let cap = captured(mov);
    if cap != EMPTY {
        add_piece(board, to, cap);
    }
    let prom = promoted(mov);
    if prom != EMPTY {
        clear_piece(board, from);
        let pawn = if PIECE_COL[prom as usize] == WHITE {
            WP
        } else {
            BP
        };
        add_piece(board, from, pawn);
    }
    debug_assert!(board.check_board());
}
