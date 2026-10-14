//! Position state: pieces, lists, and the incremental Zobrist key.

use crate::defs::*;
use crate::hash::HashTable;
use crate::tables::{set_bit, sq120, sq64, TABLES};

#[derive(Clone, Copy, Debug, Default)]
pub struct Undo {
    pub mov: u32,
    pub castle: u8,
    pub en_passant: i32,
    pub fifty_move: i32,
    pub pos_key: u64,
}

pub struct Board {
    pub pieces: [u8; SQ_NUM],
    pub pawns: [u64; 3],
    pub king_sq: [i32; 2],
    pub side: usize,
    pub en_passant: i32,
    pub fifty_move: i32,
    pub castle_perm: u8,
    pub ply: i32,
    pub history_ply: i32,
    pub pos_key: u64,
    pub piece_num: [i32; 13],
    pub big_piece: [i32; 2],
    pub major_piece: [i32; 2],
    pub minor_piece: [i32; 2],
    pub material: [i32; 2],
    pub history: [Undo; MAX_MOVES],
    pub piece_list: [[i32; 10]; 13],
    pub hash_table: HashTable,
    pub pv_array: [u32; MAX_DEPTH as usize],
    pub search_history: [[i32; SQ_NUM]; 13],
    pub search_killers: [[u32; MAX_DEPTH as usize]; 2],
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            pieces: [OFFBOARD as u8; SQ_NUM],
            pawns: [0; 3],
            king_sq: [NO_SQ; 2],
            side: BOTH,
            en_passant: NO_SQ,
            fifty_move: 0,
            castle_perm: 0,
            ply: 0,
            history_ply: 0,
            pos_key: 0,
            piece_num: [0; 13],
            big_piece: [0; 2],
            major_piece: [0; 2],
            minor_piece: [0; 2],
            material: [0; 2],
            history: [Undo::default(); MAX_MOVES],
            piece_list: [[0; 10]; 13],
            hash_table: HashTable::with_entries(1),
            pv_array: [NOMOVE; MAX_DEPTH as usize],
            search_history: [[0; SQ_NUM]; 13],
            search_killers: [[NOMOVE; MAX_DEPTH as usize]; 2],
        };
        board.reset();
        board
    }

    pub fn init_hash(&mut self, mb: usize) {
        self.hash_table = HashTable::with_mb(mb);
    }

    pub fn reset(&mut self) {
        for sq in 0..SQ_NUM {
            self.pieces[sq] = OFFBOARD as u8;
        }
        for sq64 in 0..64 {
            self.pieces[sq120(sq64) as usize] = EMPTY;
        }
        self.big_piece = [0; 2];
        self.major_piece = [0; 2];
        self.minor_piece = [0; 2];
        self.material = [0; 2];
        self.pawns = [0; 3];
        self.piece_num = [0; 13];
        self.king_sq = [NO_SQ; 2];
        self.side = BOTH;
        self.en_passant = NO_SQ;
        self.fifty_move = 0;
        self.ply = 0;
        self.history_ply = 0;
        self.castle_perm = 0;
        self.pos_key = 0;
    }

    pub fn parse_fen(&mut self, fen: &str) -> bool {
        crate::fen::parse(self, fen)
    }

    pub fn generate_pos_key(&self) -> u64 {
        let mut key = 0u64;
        for sq in 0..SQ_NUM {
            let piece = self.pieces[sq];
            if piece != OFFBOARD as u8 && piece != EMPTY {
                key ^= TABLES.piece_keys[piece as usize][sq];
            }
        }
        if self.side == WHITE {
            key ^= TABLES.side_key;
        }
        if self.en_passant != NO_SQ {
            key ^= TABLES.piece_keys[EMPTY as usize][self.en_passant as usize];
        }
        key ^= TABLES.castle_keys[self.castle_perm as usize];
        key
    }

    pub fn update_lists_material(&mut self) {
        for sq in 0..SQ_NUM as i32 {
            let piece = self.pieces[sq as usize];
            if piece == OFFBOARD as u8 || piece == EMPTY {
                continue;
            }
            let color = PIECE_COL[piece as usize];
            if PIECE_BIG[piece as usize] {
                self.big_piece[color] += 1;
            }
            if PIECE_MIN[piece as usize] {
                self.minor_piece[color] += 1;
            }
            if PIECE_MAJ[piece as usize] {
                self.major_piece[color] += 1;
            }
            self.material[color] += PIECE_VAL[piece as usize];
            let index = self.piece_num[piece as usize] as usize;
            self.piece_list[piece as usize][index] = sq;
            self.piece_num[piece as usize] += 1;
            if piece == WK {
                self.king_sq[WHITE] = sq;
            }
            if piece == BK {
                self.king_sq[BLACK] = sq;
            }
            if piece == WP {
                let s64 = sq64(sq);
                set_bit(&mut self.pawns[WHITE], s64);
                set_bit(&mut self.pawns[BOTH], s64);
            } else if piece == BP {
                let s64 = sq64(sq);
                set_bit(&mut self.pawns[BLACK], s64);
                set_bit(&mut self.pawns[BOTH], s64);
            }
        }
    }

    pub fn mirror(&mut self) {
        let temp_side = self.side ^ 1;
        let swap = [EMPTY, BP, BN, BB, BR, BQ, BK, WP, WN, WB, WR, WQ, WK];
        let mut temp_castle = 0u8;
        if (self.castle_perm & WKCA) != 0 {
            temp_castle |= BKCA;
        }
        if (self.castle_perm & WQCA) != 0 {
            temp_castle |= BQCA;
        }
        if (self.castle_perm & BKCA) != 0 {
            temp_castle |= WKCA;
        }
        if (self.castle_perm & BQCA) != 0 {
            temp_castle |= WQCA;
        }
        let temp_ep = if self.en_passant != NO_SQ {
            sq120(TABLES.mirror64[sq64(self.en_passant) as usize] as i32)
        } else {
            NO_SQ
        };
        let mut temp_pieces = [EMPTY; 64];
        for sq in 0..64 {
            let src = sq120(TABLES.mirror64[sq] as i32);
            temp_pieces[sq] = self.pieces[src as usize];
        }
        self.reset();
        for sq in 0..64 {
            let piece = swap[temp_pieces[sq] as usize];
            self.pieces[sq120(sq as i32) as usize] = piece;
        }
        self.side = temp_side;
        self.castle_perm = temp_castle;
        self.en_passant = temp_ep;
        self.pos_key = self.generate_pos_key();
        self.update_lists_material();
    }

    pub fn check_board(&self) -> bool {
        crate::validate::consistent(self)
    }

    pub(crate) fn hash_piece(&mut self, piece: u8, sq: i32) {
        self.pos_key ^= TABLES.piece_keys[piece as usize][sq as usize];
    }

    pub(crate) fn hash_castle(&mut self) {
        self.pos_key ^= TABLES.castle_keys[self.castle_perm as usize];
    }

    pub(crate) fn hash_side(&mut self) {
        self.pos_key ^= TABLES.side_key;
    }

    pub(crate) fn hash_ep(&mut self) {
        self.pos_key ^= TABLES.piece_keys[EMPTY as usize][self.en_passant as usize];
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
