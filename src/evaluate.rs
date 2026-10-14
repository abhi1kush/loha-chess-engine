//! Material, piece-square tables, and pawn-structure evaluation from the C engine.

use crate::board::Board;
use crate::defs::*;
use crate::ports::Evaluator;
use crate::tables::{file_of, rank_of, sq64, TABLES};

/// The piece-square evaluation carried over from the C engine.
pub struct ClassicEval;

impl Evaluator for ClassicEval {
    fn evaluate(&self, board: &Board) -> i32 {
        eval_position(board)
    }
}

const PAWN_ISOLATED: i32 = -10;
const PAWN_PASSED: [i32; 8] = [0, 5, 10, 20, 35, 60, 100, 200];
const ROOK_OPEN_FILE: i32 = 10;
const ROOK_SEMI_OPEN_FILE: i32 = 5;
const QUEEN_OPEN_FILE: i32 = 5;
const QUEEN_SEMI_OPEN_FILE: i32 = 3;
const BISHOP_PAIR: i32 = 30;

const PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 0, -10, -10, 0, 10, 10, 5, 0, 0, 5, 5, 0, 0, 5, 0, 0, 10, 20,
    20, 10, 0, 0, 5, 5, 5, 10, 10, 5, 5, 5, 10, 10, 10, 20, 20, 10, 10, 10, 20, 20, 20, 30, 30, 20,
    20, 20, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_TABLE: [i32; 64] = [
    0, -10, 0, 0, 0, 0, -10, 0, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 10, 10, 10, 10, 0, 0, 0, 0, 10, 20,
    20, 10, 5, 0, 5, 10, 15, 20, 20, 15, 10, 5, 5, 10, 10, 20, 20, 10, 10, 5, 0, 0, 5, 10, 10, 5,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const BISHOP_TABLE: [i32; 64] = [
    0, 0, -10, 0, 0, -10, 0, 0, 0, 0, 0, 10, 10, 0, 0, 0, 0, 0, 10, 15, 15, 10, 0, 0, 0, 10, 15,
    20, 20, 15, 10, 0, 0, 10, 15, 20, 20, 15, 10, 0, 0, 0, 10, 15, 15, 10, 0, 0, 0, 0, 0, 10, 10,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const ROOK_TABLE: [i32; 64] = [
    0, 0, 5, 10, 10, 5, 0, 0, 0, 0, 5, 10, 10, 5, 0, 0, 0, 0, 5, 10, 10, 5, 0, 0, 0, 0, 5, 10, 10,
    5, 0, 0, 0, 0, 5, 10, 10, 5, 0, 0, 0, 0, 5, 10, 10, 5, 0, 0, 25, 25, 25, 25, 25, 25, 25, 25, 0,
    0, 5, 10, 10, 5, 0, 0,
];

const KING_END: [i32; 64] = [
    -50, -10, 0, 0, 0, 0, -10, -50, -10, 0, 10, 10, 10, 10, 0, -10, 0, 10, 20, 20, 20, 20, 10, 0,
    0, 10, 20, 40, 40, 20, 10, 0, 0, 10, 20, 40, 40, 20, 10, 0, 0, 10, 20, 20, 20, 20, 10, 0, -10,
    0, 10, 10, 10, 10, 0, -10, -50, -10, 0, 0, 0, 0, -10, -50,
];

const KING_OPEN: [i32; 64] = [
    0, 5, 5, -10, -10, 0, 10, 5, -30, -30, -30, -30, -30, -30, -30, -30, -50, -50, -50, -50, -50,
    -50, -50, -50, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70,
    -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70, -70,
    -70, -70, -70, -70, -70,
];

fn material_draw(board: &Board) -> bool {
    if board.piece_num[WR as usize] == 0
        && board.piece_num[BR as usize] == 0
        && board.piece_num[WQ as usize] == 0
        && board.piece_num[BQ as usize] == 0
    {
        if board.piece_num[BB as usize] == 0 && board.piece_num[WB as usize] == 0 {
            if board.piece_num[WN as usize] < 3 && board.piece_num[BN as usize] < 3 {
                return true;
            }
        } else if board.piece_num[WN as usize] == 0 && board.piece_num[BN as usize] == 0 {
            if (board.piece_num[WB as usize] - board.piece_num[BB as usize]).abs() < 2 {
                return true;
            }
        } else if (board.piece_num[WN as usize] < 3 && board.piece_num[WB as usize] == 0)
            || (board.piece_num[WB as usize] == 1 && board.piece_num[WN as usize] == 0)
        {
            if (board.piece_num[BN as usize] < 3 && board.piece_num[BB as usize] == 0)
                || (board.piece_num[BB as usize] == 1 && board.piece_num[BN as usize] == 0)
            {
                return true;
            }
        }
    } else if board.piece_num[WQ as usize] == 0 && board.piece_num[BQ as usize] == 0 {
        if board.piece_num[WR as usize] == 1 && board.piece_num[BR as usize] == 1 {
            if (board.piece_num[WN as usize] + board.piece_num[WB as usize]) < 2
                && (board.piece_num[BN as usize] + board.piece_num[BB as usize]) < 2
            {
                return true;
            }
        } else if board.piece_num[WR as usize] == 1 && board.piece_num[BR as usize] == 0 {
            let black_minor = board.piece_num[BN as usize] + board.piece_num[BB as usize];
            if board.piece_num[WN as usize] + board.piece_num[WB as usize] == 0
                && (black_minor == 1 || black_minor == 2)
            {
                return true;
            }
        } else if board.piece_num[BR as usize] == 1 && board.piece_num[WR as usize] == 0 {
            let white_minor = board.piece_num[WN as usize] + board.piece_num[WB as usize];
            if board.piece_num[BN as usize] + board.piece_num[BB as usize] == 0
                && (white_minor == 1 || white_minor == 2)
            {
                return true;
            }
        }
    }
    false
}

pub fn eval_position(board: &Board) -> i32 {
    if board.piece_num[WP as usize] == 0
        && board.piece_num[BP as usize] == 0
        && material_draw(board)
    {
        return 0;
    }

    let mut score = board.material[WHITE] - board.material[BLACK];

    for n in 0..board.piece_num[WP as usize] {
        let sq = board.piece_list[WP as usize][n as usize];
        let s64 = sq64(sq) as usize;
        score += PAWN_TABLE[s64];
        if TABLES.isolated_mask[s64] & board.pawns[WHITE] == 0 {
            score += PAWN_ISOLATED;
        }
        if TABLES.white_passed_mask[s64] & board.pawns[BLACK] == 0 {
            score += PAWN_PASSED[rank_of(sq) as usize];
        }
    }

    for n in 0..board.piece_num[BP as usize] {
        let sq = board.piece_list[BP as usize][n as usize];
        let s64 = sq64(sq) as usize;
        score -= PAWN_TABLE[TABLES.mirror64[s64]];
        if TABLES.isolated_mask[s64] & board.pawns[BLACK] == 0 {
            score -= PAWN_ISOLATED;
        }
        if TABLES.black_passed_mask[s64] & board.pawns[WHITE] == 0 {
            score -= PAWN_PASSED[(7 - rank_of(sq)) as usize];
        }
    }

    for n in 0..board.piece_num[WN as usize] {
        let sq = board.piece_list[WN as usize][n as usize];
        score += KNIGHT_TABLE[sq64(sq) as usize];
    }
    for n in 0..board.piece_num[BN as usize] {
        let sq = board.piece_list[BN as usize][n as usize];
        score -= KNIGHT_TABLE[TABLES.mirror64[sq64(sq) as usize]];
    }

    for n in 0..board.piece_num[WR as usize] {
        let sq = board.piece_list[WR as usize][n as usize];
        let s64 = sq64(sq) as usize;
        score += ROOK_TABLE[s64];
        let file_mask = TABLES.file_bb_mask[file_of(sq) as usize];
        if board.pawns[BOTH] & file_mask == 0 {
            score += ROOK_OPEN_FILE;
        } else if board.pawns[WHITE] & file_mask == 0 {
            score += ROOK_SEMI_OPEN_FILE;
        }
    }
    for n in 0..board.piece_num[BR as usize] {
        let sq = board.piece_list[BR as usize][n as usize];
        score -= ROOK_TABLE[TABLES.mirror64[sq64(sq) as usize]];
        let file_mask = TABLES.file_bb_mask[file_of(sq) as usize];
        if board.pawns[BOTH] & file_mask == 0 {
            score -= ROOK_OPEN_FILE;
        } else if board.pawns[BLACK] & file_mask == 0 {
            score -= ROOK_SEMI_OPEN_FILE;
        }
    }

    for n in 0..board.piece_num[WB as usize] {
        let sq = board.piece_list[WB as usize][n as usize];
        score += BISHOP_TABLE[sq64(sq) as usize];
    }
    for n in 0..board.piece_num[BB as usize] {
        let sq = board.piece_list[BB as usize][n as usize];
        score -= BISHOP_TABLE[TABLES.mirror64[sq64(sq) as usize]];
    }

    for n in 0..board.piece_num[WQ as usize] {
        let sq = board.piece_list[WQ as usize][n as usize];
        let file_mask = TABLES.file_bb_mask[file_of(sq) as usize];
        if board.pawns[BOTH] & file_mask == 0 {
            score += QUEEN_OPEN_FILE;
        } else if board.pawns[WHITE] & file_mask == 0 {
            score += QUEEN_SEMI_OPEN_FILE;
        }
    }
    for n in 0..board.piece_num[BQ as usize] {
        let sq = board.piece_list[BQ as usize][n as usize];
        let file_mask = TABLES.file_bb_mask[file_of(sq) as usize];
        if board.pawns[BOTH] & file_mask == 0 {
            score -= QUEEN_OPEN_FILE;
        } else if board.pawns[BLACK] & file_mask == 0 {
            score -= QUEEN_SEMI_OPEN_FILE;
        }
    }

    let white_king = sq64(board.piece_list[WK as usize][0]) as usize;
    if board.material[BLACK] <= ENDGAME_MAT {
        score += KING_END[white_king];
    } else {
        score += KING_OPEN[white_king];
    }
    let black_king = sq64(board.piece_list[BK as usize][0]) as usize;
    if board.material[WHITE] <= ENDGAME_MAT {
        score -= KING_END[TABLES.mirror64[black_king]];
    } else {
        score -= KING_OPEN[TABLES.mirror64[black_king]];
    }

    if board.piece_num[WB as usize] >= 2 {
        score += BISHOP_PAIR;
    }
    if board.piece_num[BB as usize] >= 2 {
        score -= BISHOP_PAIR;
    }

    if board.side == WHITE {
        score
    } else {
        -score
    }
}
