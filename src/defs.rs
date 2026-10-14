//! Shared constants, piece codes, and move packing.
//!
//! Moves use the same bit layout as the C engine:
//! bits  0..6  from square (0x7F)
//! bits  7..13 to square
//! bits 14..17 captured piece
//! bit  18     en passant
//! bit  19     double pawn push
//! bits 20..23 promoted piece
//! bit  24     castle

pub const NAME: &str = "loha 1.0";
pub const AUTHOR: &str = "Abhishek Pratap Singh";

pub const SQ_NUM: usize = 120;
pub const MAX_MOVES: usize = 2048;
pub const MAX_POSITION_MOVES: usize = 256;
pub const MAX_DEPTH: i32 = 64;
pub const MAX_HASH_MB: usize = 2048;

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const INFINITE: i32 = 30_000;
pub const IS_MATE: i32 = INFINITE - MAX_DEPTH;
pub const MATE_SCORE: i32 = 29_000;

pub const EMPTY: u8 = 0;
pub const WP: u8 = 1;
pub const WN: u8 = 2;
pub const WB: u8 = 3;
pub const WR: u8 = 4;
pub const WQ: u8 = 5;
pub const WK: u8 = 6;
pub const BP: u8 = 7;
pub const BN: u8 = 8;
pub const BB: u8 = 9;
pub const BR: u8 = 10;
pub const BQ: u8 = 11;
pub const BK: u8 = 12;

pub const FILE_A: i32 = 0;
pub const FILE_H: i32 = 7;
pub const RANK_1: i32 = 0;
pub const RANK_2: i32 = 1;
pub const RANK_3: i32 = 2;
pub const RANK_6: i32 = 5;
pub const RANK_7: i32 = 6;
pub const RANK_8: i32 = 7;

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
pub const BOTH: usize = 2;

pub const WKCA: u8 = 1;
pub const WQCA: u8 = 2;
pub const BKCA: u8 = 4;
pub const BQCA: u8 = 8;

pub const A1: i32 = 21;
pub const B1: i32 = 22;
pub const C1: i32 = 23;
pub const D1: i32 = 24;
pub const E1: i32 = 25;
pub const F1: i32 = 26;
pub const G1: i32 = 27;
pub const H1: i32 = 28;
pub const A8: i32 = 91;
pub const B8: i32 = 92;
pub const C8: i32 = 93;
pub const D8: i32 = 94;
pub const E8: i32 = 95;
pub const F8: i32 = 96;
pub const G8: i32 = 97;
pub const H8: i32 = 98;
pub const NO_SQ: i32 = 99;
pub const OFFBOARD: i32 = 100;

pub const HF_NONE: u8 = 0;
pub const HF_ALPHA: u8 = 1;
pub const HF_BETA: u8 = 2;
pub const HF_EXACT: u8 = 3;

pub const MFLAG_EP: u32 = 0x40000;
pub const MFLAG_PS: u32 = 0x80000;
pub const MFLAG_CA: u32 = 0x0100_0000;
pub const MFLAG_CAP: u32 = 0x7C000;
pub const NOMOVE: u32 = 0;

pub const PIECE_CHAR: &[u8] = b".PNBRQKpnbrqk";
pub const SIDE_CHAR: &[u8] = b"wb-";
pub const FILE_CHAR: &[u8] = b"abcdefgh";
pub const RANK_CHAR: &[u8] = b"12345678";

pub const PIECE_BIG: [bool; 13] = [
    false, false, true, true, true, true, true, false, true, true, true, true, true,
];
pub const PIECE_MAJ: [bool; 13] = [
    false, false, false, false, true, true, true, false, false, false, true, true, true,
];
pub const PIECE_MIN: [bool; 13] = [
    false, false, true, true, false, false, false, false, true, true, false, false, false,
];
pub const PIECE_VAL: [i32; 13] = [
    0, 100, 325, 325, 550, 1000, 50000, 100, 325, 325, 550, 1000, 50000,
];
pub const PIECE_COL: [usize; 13] = [
    BOTH, WHITE, WHITE, WHITE, WHITE, WHITE, WHITE, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK,
];
pub const PIECE_PAWN: [bool; 13] = [
    false, true, false, false, false, false, false, true, false, false, false, false, false,
];
pub const PIECE_KNIGHT: [bool; 13] = [
    false, false, true, false, false, false, false, false, true, false, false, false, false,
];
pub const PIECE_KING: [bool; 13] = [
    false, false, false, false, false, false, true, false, false, false, false, false, true,
];
pub const PIECE_RQ: [bool; 13] = [
    false, false, false, false, true, true, false, false, false, false, true, true, false,
];
pub const PIECE_BQ: [bool; 13] = [
    false, false, false, true, false, true, false, false, false, true, false, true, false,
];

pub const ENDGAME_MAT: i32 =
    PIECE_VAL[WR as usize] + 2 * PIECE_VAL[WN as usize] + 2 * PIECE_VAL[WP as usize];

#[inline]
pub const fn fr2sq(file: i32, rank: i32) -> i32 {
    21 + file + rank * 10
}

#[inline]
pub const fn from_sq(mov: u32) -> i32 {
    (mov & 0x7F) as i32
}

#[inline]
pub const fn to_sq(mov: u32) -> i32 {
    ((mov >> 7) & 0x7F) as i32
}

#[inline]
pub const fn captured(mov: u32) -> u8 {
    ((mov >> 14) & 0xF) as u8
}

#[inline]
pub const fn promoted(mov: u32) -> u8 {
    ((mov >> 20) & 0xF) as u8
}

#[inline]
pub const fn encode_move(from: i32, to: i32, cap: u8, prom: u8, flag: u32) -> u32 {
    (from as u32) | ((to as u32) << 7) | ((cap as u32) << 14) | ((prom as u32) << 20) | flag
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Move {
    pub mov: u32,
    pub score: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct MoveList {
    pub moves: [Move; MAX_POSITION_MOVES],
    pub count: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self {
            moves: [Move::default(); MAX_POSITION_MOVES],
            count: 0,
        }
    }
}
