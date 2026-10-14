//! Square conversion, Zobrist keys, and evaluation masks.
//! Built once at compile time so the engine has no init-order dependency.

use crate::defs::*;

pub struct Tables {
    pub sq120_to_sq64: [i32; SQ_NUM],
    pub sq64_to_sq120: [i32; 64],
    pub files_brd: [i32; SQ_NUM],
    pub ranks_brd: [i32; SQ_NUM],
    pub file_bb_mask: [u64; 8],
    pub rank_bb_mask: [u64; 8],
    pub black_passed_mask: [u64; 64],
    pub white_passed_mask: [u64; 64],
    pub isolated_mask: [u64; 64],
    pub piece_keys: [[u64; SQ_NUM]; 13],
    pub side_key: u64,
    pub castle_keys: [u64; 16],
    pub mirror64: [usize; 64],
}

pub static TABLES: Tables = Tables::new();

impl Tables {
    const fn new() -> Self {
        let mut sq120_to_sq64 = [65; SQ_NUM];
        let mut sq64_to_sq120 = [120; 64];
        let mut files_brd = [OFFBOARD; SQ_NUM];
        let mut ranks_brd = [OFFBOARD; SQ_NUM];

        let mut rank = RANK_1;
        while rank <= RANK_8 {
            let mut file = FILE_A;
            while file <= FILE_H {
                let sq = fr2sq(file, rank);
                let sq64 = rank * 8 + file;
                sq64_to_sq120[sq64 as usize] = sq;
                sq120_to_sq64[sq as usize] = sq64;
                files_brd[sq as usize] = file;
                ranks_brd[sq as usize] = rank;
                file += 1;
            }
            rank += 1;
        }

        let mut file_bb_mask = [0u64; 8];
        let mut rank_bb_mask = [0u64; 8];
        rank = RANK_8;
        loop {
            let mut file = FILE_A;
            while file <= FILE_H {
                let sq = (rank * 8 + file) as usize;
                file_bb_mask[file as usize] |= 1u64 << sq;
                rank_bb_mask[rank as usize] |= 1u64 << sq;
                file += 1;
            }
            if rank == RANK_1 {
                break;
            }
            rank -= 1;
        }

        let mut isolated_mask = [0u64; 64];
        let mut white_passed_mask = [0u64; 64];
        let mut black_passed_mask = [0u64; 64];

        let mut sq = 0usize;
        while sq < 64 {
            let mut tsq = sq + 8;
            while tsq < 64 {
                white_passed_mask[sq] |= 1u64 << tsq;
                tsq += 8;
            }
            tsq = sq.wrapping_sub(8);
            while tsq < 64 {
                black_passed_mask[sq] |= 1u64 << tsq;
                if tsq < 8 {
                    break;
                }
                tsq -= 8;
            }

            let file = files_brd[sq64_to_sq120[sq] as usize];
            if file > FILE_A {
                isolated_mask[sq] |= file_bb_mask[(file - 1) as usize];
                tsq = sq + 7;
                while tsq < 64 {
                    white_passed_mask[sq] |= 1u64 << tsq;
                    tsq += 8;
                }
                tsq = sq.wrapping_sub(9);
                while tsq < 64 {
                    black_passed_mask[sq] |= 1u64 << tsq;
                    if tsq < 8 {
                        break;
                    }
                    tsq -= 8;
                }
            }
            if file < FILE_H {
                isolated_mask[sq] |= file_bb_mask[(file + 1) as usize];
                tsq = sq + 9;
                while tsq < 64 {
                    white_passed_mask[sq] |= 1u64 << tsq;
                    tsq += 8;
                }
                tsq = sq.wrapping_sub(7);
                while tsq < 64 {
                    black_passed_mask[sq] |= 1u64 << tsq;
                    if tsq < 8 {
                        break;
                    }
                    tsq -= 8;
                }
            }
            sq += 1;
        }

        let mut state = 0xC0FFEE_u64 ^ 0x1234_5678_9ABC_DEF0;
        let side_key = xorshift(&mut state);
        let mut piece_keys = [[0u64; SQ_NUM]; 13];
        let mut p = 0;
        while p < 13 {
            let mut s = 0;
            while s < SQ_NUM {
                piece_keys[p][s] = xorshift(&mut state);
                s += 1;
            }
            p += 1;
        }
        let mut castle_keys = [0u64; 16];
        let mut c = 0;
        while c < 16 {
            castle_keys[c] = xorshift(&mut state);
            c += 1;
        }

        let mut mirror64 = [0usize; 64];
        let mut i = 0;
        while i < 64 {
            mirror64[i] = (i ^ 56) as usize;
            i += 1;
        }

        Self {
            sq120_to_sq64,
            sq64_to_sq120,
            files_brd,
            ranks_brd,
            file_bb_mask,
            rank_bb_mask,
            black_passed_mask,
            white_passed_mask,
            isolated_mask,
            piece_keys,
            side_key,
            castle_keys,
            mirror64,
        }
    }
}

const fn xorshift(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

#[inline]
pub fn sq64(sq120: i32) -> i32 {
    TABLES.sq120_to_sq64[sq120 as usize]
}

#[inline]
pub fn sq120(sq64: i32) -> i32 {
    TABLES.sq64_to_sq120[sq64 as usize]
}

#[inline]
pub fn file_of(sq120: i32) -> i32 {
    TABLES.files_brd[sq120 as usize]
}

#[inline]
pub fn rank_of(sq120: i32) -> i32 {
    TABLES.ranks_brd[sq120 as usize]
}

#[inline]
pub fn sq_offboard(sq: i32) -> bool {
    sq < 0 || sq as usize >= SQ_NUM || TABLES.files_brd[sq as usize] == OFFBOARD
}

#[inline]
pub fn sq_on_board(sq: i32) -> bool {
    !sq_offboard(sq)
}

#[inline]
pub fn set_bit(bb: &mut u64, sq64: i32) {
    *bb |= 1u64 << sq64;
}

#[inline]
pub fn clear_bit(bb: &mut u64, sq64: i32) {
    *bb &= !(1u64 << sq64);
}
