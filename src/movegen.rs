//! Legal-ish move generation. Moves that leave the king in check are filtered later
//! by `make_move`, matching the C engine.

use crate::attack::sq_attacked;
use crate::board::Board;
use crate::defs::*;
use crate::makemove::{make_move, take_move};
use crate::tables::{file_of, rank_of};

const LOOP_SLIDE_PCE: [u8; 8] = [WB, WR, WQ, 0, BB, BR, BQ, 0];
const LOOP_SLIDE_INDEX: [usize; 2] = [0, 4];
const LOOP_NONSLIDE_PCE: [u8; 6] = [WN, WK, 0, BN, BK, 0];
const LOOP_NONSLIDE_INDEX: [usize; 2] = [0, 3];

const PIECE_DIR: [[i32; 8]; 13] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-8, -19, -21, -12, 8, 19, 21, 12],
    [-9, -11, 11, 9, 0, 0, 0, 0],
    [-1, -10, 1, 10, 0, 0, 0, 0],
    [-1, -10, 1, 10, -9, -11, 11, 9],
    [-1, -10, 1, 10, -9, -11, 11, 9],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [-8, -19, -21, -12, 8, 19, 21, 12],
    [-9, -11, 11, 9, 0, 0, 0, 0],
    [-1, -10, 1, 10, 0, 0, 0, 0],
    [-1, -10, 1, 10, -9, -11, 11, 9],
    [-1, -10, 1, 10, -9, -11, 11, 9],
];

const NUM_DIR: [usize; 13] = [0, 0, 8, 4, 4, 8, 8, 0, 8, 4, 4, 8, 8];

const VICTIM_SCORE: [i32; 13] = [
    0, 100, 200, 300, 400, 500, 600, 100, 200, 300, 400, 500, 600,
];

const fn mvv_lva() -> [[i32; 13]; 13] {
    let mut scores = [[0i32; 13]; 13];
    let mut attacker = 1;
    while attacker <= 12 {
        let mut victim = 1;
        while victim <= 12 {
            scores[victim][attacker] = VICTIM_SCORE[victim] + 6 - (VICTIM_SCORE[attacker] / 100);
            victim += 1;
        }
        attacker += 1;
    }
    scores
}

const MVV_LVA: [[i32; 13]; 13] = mvv_lva();

fn add_quiet(board: &Board, mov: u32, list: &mut MoveList) {
    let ply = board.ply as usize;
    let score = if board.search_killers[0][ply] == mov {
        900_000
    } else if board.search_killers[1][ply] == mov {
        800_000
    } else {
        let piece = board.pieces[from_sq(mov) as usize];
        board.search_history[piece as usize][to_sq(mov) as usize]
    };
    list.moves[list.count] = Move { mov, score };
    list.count += 1;
}

fn add_capture(board: &Board, mov: u32, list: &mut MoveList) {
    let victim = captured(mov) as usize;
    let attacker = board.pieces[from_sq(mov) as usize] as usize;
    let score = MVV_LVA[victim][attacker] + 1_000_000;
    list.moves[list.count] = Move { mov, score };
    list.count += 1;
}

fn add_en_passant(mov: u32, list: &mut MoveList) {
    list.moves[list.count] = Move {
        mov,
        score: 105 + 1_000_000,
    };
    list.count += 1;
}

fn add_white_pawn_cap(board: &Board, from: i32, to: i32, cap: u8, list: &mut MoveList) {
    if rank_of(from) == RANK_7 {
        add_capture(board, encode_move(from, to, cap, WQ, 0), list);
        add_capture(board, encode_move(from, to, cap, WR, 0), list);
        add_capture(board, encode_move(from, to, cap, WB, 0), list);
        add_capture(board, encode_move(from, to, cap, WN, 0), list);
    } else {
        add_capture(board, encode_move(from, to, cap, EMPTY, 0), list);
    }
}

fn add_white_pawn_move(board: &Board, from: i32, to: i32, list: &mut MoveList) {
    if rank_of(from) == RANK_7 {
        add_quiet(board, encode_move(from, to, EMPTY, WQ, 0), list);
        add_quiet(board, encode_move(from, to, EMPTY, WR, 0), list);
        add_quiet(board, encode_move(from, to, EMPTY, WB, 0), list);
        add_quiet(board, encode_move(from, to, EMPTY, WN, 0), list);
    } else {
        add_quiet(board, encode_move(from, to, EMPTY, EMPTY, 0), list);
    }
}

fn add_black_pawn_cap(board: &Board, from: i32, to: i32, cap: u8, list: &mut MoveList) {
    if rank_of(from) == RANK_2 {
        add_capture(board, encode_move(from, to, cap, BQ, 0), list);
        add_capture(board, encode_move(from, to, cap, BR, 0), list);
        add_capture(board, encode_move(from, to, cap, BB, 0), list);
        add_capture(board, encode_move(from, to, cap, BN, 0), list);
    } else {
        add_capture(board, encode_move(from, to, cap, EMPTY, 0), list);
    }
}

fn add_black_pawn_move(board: &Board, from: i32, to: i32, list: &mut MoveList) {
    if rank_of(from) == RANK_2 {
        add_quiet(board, encode_move(from, to, EMPTY, BQ, 0), list);
        add_quiet(board, encode_move(from, to, EMPTY, BR, 0), list);
        add_quiet(board, encode_move(from, to, EMPTY, BB, 0), list);
        add_quiet(board, encode_move(from, to, EMPTY, BN, 0), list);
    } else {
        add_quiet(board, encode_move(from, to, EMPTY, EMPTY, 0), list);
    }
}

pub fn generate_all_moves(board: &Board, list: &mut MoveList) {
    generate(board, list, false);
}

pub fn generate_all_caps(board: &Board, list: &mut MoveList) {
    generate(board, list, true);
}

fn generate(board: &Board, list: &mut MoveList, caps_only: bool) {
    list.count = 0;
    let side = board.side;

    if side == WHITE {
        for n in 0..board.piece_num[WP as usize] {
            let sq = board.piece_list[WP as usize][n as usize];
            if !caps_only && board.pieces[(sq + 10) as usize] == EMPTY {
                add_white_pawn_move(board, sq, sq + 10, list);
                if rank_of(sq) == RANK_2 && board.pieces[(sq + 20) as usize] == EMPTY {
                    add_quiet(
                        board,
                        encode_move(sq, sq + 20, EMPTY, EMPTY, MFLAG_PS),
                        list,
                    );
                }
            }
            if !file_off(sq + 9) && PIECE_COL[board.pieces[(sq + 9) as usize] as usize] == BLACK {
                add_white_pawn_cap(board, sq, sq + 9, board.pieces[(sq + 9) as usize], list);
            }
            if !file_off(sq + 11) && PIECE_COL[board.pieces[(sq + 11) as usize] as usize] == BLACK {
                add_white_pawn_cap(board, sq, sq + 11, board.pieces[(sq + 11) as usize], list);
            }
            if !file_off(sq + 9) && sq + 9 == board.en_passant {
                add_en_passant(encode_move(sq, sq + 9, EMPTY, EMPTY, MFLAG_EP), list);
            }
            if !file_off(sq + 11) && sq + 11 == board.en_passant {
                add_en_passant(encode_move(sq, sq + 11, EMPTY, EMPTY, MFLAG_EP), list);
            }
        }
        if !caps_only
            && (board.castle_perm & WKCA) != 0
            && board.pieces[F1 as usize] == EMPTY
            && board.pieces[G1 as usize] == EMPTY
            && !sq_attacked(board, E1, BLACK)
            && !sq_attacked(board, F1, BLACK)
            && !sq_attacked(board, G1, BLACK)
        {
            add_quiet(board, encode_move(E1, G1, EMPTY, EMPTY, MFLAG_CA), list);
        }
        if !caps_only
            && (board.castle_perm & WQCA) != 0
            && board.pieces[D1 as usize] == EMPTY
            && board.pieces[C1 as usize] == EMPTY
            && board.pieces[B1 as usize] == EMPTY
            && !sq_attacked(board, E1, BLACK)
            && !sq_attacked(board, D1, BLACK)
            && !sq_attacked(board, C1, BLACK)
        {
            add_quiet(board, encode_move(E1, C1, EMPTY, EMPTY, MFLAG_CA), list);
        }
    } else {
        for n in 0..board.piece_num[BP as usize] {
            let sq = board.piece_list[BP as usize][n as usize];
            if !caps_only && board.pieces[(sq - 10) as usize] == EMPTY {
                add_black_pawn_move(board, sq, sq - 10, list);
                if rank_of(sq) == RANK_7 && board.pieces[(sq - 20) as usize] == EMPTY {
                    add_quiet(
                        board,
                        encode_move(sq, sq - 20, EMPTY, EMPTY, MFLAG_PS),
                        list,
                    );
                }
            }
            if !file_off(sq - 9) && PIECE_COL[board.pieces[(sq - 9) as usize] as usize] == WHITE {
                add_black_pawn_cap(board, sq, sq - 9, board.pieces[(sq - 9) as usize], list);
            }
            if !file_off(sq - 11) && PIECE_COL[board.pieces[(sq - 11) as usize] as usize] == WHITE {
                add_black_pawn_cap(board, sq, sq - 11, board.pieces[(sq - 11) as usize], list);
            }
            if !file_off(sq - 9) && sq - 9 == board.en_passant {
                add_en_passant(encode_move(sq, sq - 9, EMPTY, EMPTY, MFLAG_EP), list);
            }
            if !file_off(sq - 11) && sq - 11 == board.en_passant {
                add_en_passant(encode_move(sq, sq - 11, EMPTY, EMPTY, MFLAG_EP), list);
            }
        }
        if !caps_only
            && (board.castle_perm & BKCA) != 0
            && board.pieces[F8 as usize] == EMPTY
            && board.pieces[G8 as usize] == EMPTY
            && !sq_attacked(board, E8, WHITE)
            && !sq_attacked(board, F8, WHITE)
            && !sq_attacked(board, G8, WHITE)
        {
            add_quiet(board, encode_move(E8, G8, EMPTY, EMPTY, MFLAG_CA), list);
        }
        if !caps_only
            && (board.castle_perm & BQCA) != 0
            && board.pieces[D8 as usize] == EMPTY
            && board.pieces[C8 as usize] == EMPTY
            && board.pieces[B8 as usize] == EMPTY
            && !sq_attacked(board, E8, WHITE)
            && !sq_attacked(board, D8, WHITE)
            && !sq_attacked(board, C8, WHITE)
        {
            add_quiet(board, encode_move(E8, C8, EMPTY, EMPTY, MFLAG_CA), list);
        }
    }

    let mut index = LOOP_SLIDE_INDEX[side];
    let mut piece = LOOP_SLIDE_PCE[index];
    index += 1;
    while piece != 0 {
        for n in 0..board.piece_num[piece as usize] {
            let sq = board.piece_list[piece as usize][n as usize];
            for d in 0..NUM_DIR[piece as usize] {
                let dir = PIECE_DIR[piece as usize][d];
                let mut t_sq = sq + dir;
                while !file_off(t_sq) {
                    let occupant = board.pieces[t_sq as usize];
                    if occupant != EMPTY {
                        if PIECE_COL[occupant as usize] == side ^ 1 {
                            add_capture(board, encode_move(sq, t_sq, occupant, EMPTY, 0), list);
                        }
                        break;
                    }
                    if !caps_only {
                        add_quiet(board, encode_move(sq, t_sq, EMPTY, EMPTY, 0), list);
                    }
                    t_sq += dir;
                }
            }
        }
        piece = LOOP_SLIDE_PCE[index];
        index += 1;
    }

    let mut index = LOOP_NONSLIDE_INDEX[side];
    let mut piece = LOOP_NONSLIDE_PCE[index];
    index += 1;
    while piece != 0 {
        for n in 0..board.piece_num[piece as usize] {
            let sq = board.piece_list[piece as usize][n as usize];
            for d in 0..NUM_DIR[piece as usize] {
                let dir = PIECE_DIR[piece as usize][d];
                let t_sq = sq + dir;
                if file_off(t_sq) {
                    continue;
                }
                let occupant = board.pieces[t_sq as usize];
                if occupant != EMPTY {
                    if PIECE_COL[occupant as usize] == side ^ 1 {
                        add_capture(board, encode_move(sq, t_sq, occupant, EMPTY, 0), list);
                    }
                    continue;
                }
                if !caps_only {
                    add_quiet(board, encode_move(sq, t_sq, EMPTY, EMPTY, 0), list);
                }
            }
        }
        piece = LOOP_NONSLIDE_PCE[index];
        index += 1;
    }
}

fn file_off(sq: i32) -> bool {
    file_of(sq) == OFFBOARD
}

pub fn move_exists(board: &mut Board, mov: u32) -> bool {
    let mut list = MoveList::default();
    generate_all_moves(board, &mut list);
    for n in 0..list.count {
        if !make_move(board, list.moves[n].mov) {
            continue;
        }
        take_move(board);
        if list.moves[n].mov == mov {
            return true;
        }
    }
    false
}
