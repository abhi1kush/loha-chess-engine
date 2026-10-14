//! Polyglot opening book. The file format stays here; search only asks for a move.

use crate::board::Board;
use crate::defs::*;
use crate::notation::parse_move;
use crate::polykeys::POLY_KEYS;
use crate::ports::{BookSwitch, OpeningBook};
use crate::tables::{file_of, rank_of};

const POLY_KIND: [i32; 13] = [-1, 1, 3, 5, 7, 9, 11, 0, 2, 4, 6, 8, 10];

struct BookEntry {
    key: u64,
    mov: u16,
}

pub struct PolyglotBook {
    entries: Vec<BookEntry>,
    enabled: bool,
    rng: u64,
}

/// A book that never suggests a move. Protocols can use it in place of a file.
pub struct EmptyBook;

impl PolyglotBook {
    pub fn load() -> Self {
        let mut book = Self {
            entries: Vec::new(),
            enabled: false,
            rng: 0x1234_5678_9ABC_DEF0,
        };
        let data = match std::fs::read("performance.bin") {
            Ok(data) => data,
            Err(_) => {
                println!("Book File Not Read");
                return book;
            }
        };
        const ENTRY: usize = 16;
        if data.len() < ENTRY {
            println!("No Entries Found");
            return book;
        }
        for chunk in data.chunks_exact(ENTRY) {
            let key = u64::from_be_bytes(chunk[0..8].try_into().unwrap());
            let mov = u16::from_be_bytes(chunk[8..10].try_into().unwrap());
            book.entries.push(BookEntry { key, mov });
        }
        println!("{} Entries Found In File", book.entries.len());
        println!("fread() {} Entries Read in from file", book.entries.len());
        book.enabled = !book.entries.is_empty();
        book
    }
}

impl OpeningBook for PolyglotBook {
    fn suggest(&mut self, board: &Board) -> u32 {
        if !self.enabled || self.entries.is_empty() {
            return NOMOVE;
        }
        let key = poly_key_from_board(board);
        let mut moves = Vec::new();
        for entry in &self.entries {
            if entry.key != key {
                continue;
            }
            let mov = convert_poly_move(entry.mov, board);
            if mov != NOMOVE {
                moves.push(mov);
                if moves.len() > 32 {
                    break;
                }
            }
        }
        if moves.is_empty() {
            NOMOVE
        } else {
            moves[(self.next_rand() as usize) % moves.len()]
        }
    }
}

impl BookSwitch for PolyglotBook {
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl OpeningBook for EmptyBook {
    fn suggest(&mut self, _board: &Board) -> u32 {
        NOMOVE
    }
}

impl BookSwitch for EmptyBook {
    fn set_enabled(&mut self, _enabled: bool) {}
}

impl PolyglotBook {
    fn next_rand(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        if x == 0 {
            x = 0x9E37_79B9_7F4A_7C15;
        }
        self.rng = x;
        x
    }
}

fn has_pawn_for_capture(board: &Board) -> bool {
    if board.en_passant == NO_SQ {
        return false;
    }
    let target = if board.side == WHITE { WP } else { BP };
    let sq_with_pawn = if board.side == WHITE {
        board.en_passant - 10
    } else {
        board.en_passant + 10
    };
    board.pieces[(sq_with_pawn + 1) as usize] == target
        || board.pieces[(sq_with_pawn - 1) as usize] == target
}

fn poly_key_from_board(board: &Board) -> u64 {
    let mut key = 0u64;
    for sq in 0..SQ_NUM as i32 {
        let piece = board.pieces[sq as usize];
        if piece == EMPTY || piece == OFFBOARD as u8 {
            continue;
        }
        let poly_piece = POLY_KIND[piece as usize];
        let rank = rank_of(sq);
        let file = file_of(sq);
        key ^= POLY_KEYS[(64 * poly_piece + 8 * rank + file) as usize];
    }
    let offset = 768usize;
    if (board.castle_perm & WKCA) != 0 {
        key ^= POLY_KEYS[offset];
    }
    if (board.castle_perm & WQCA) != 0 {
        key ^= POLY_KEYS[offset + 1];
    }
    if (board.castle_perm & BKCA) != 0 {
        key ^= POLY_KEYS[offset + 2];
    }
    if (board.castle_perm & BQCA) != 0 {
        key ^= POLY_KEYS[offset + 3];
    }
    if has_pawn_for_capture(board) {
        key ^= POLY_KEYS[772 + file_of(board.en_passant) as usize];
    }
    if board.side == WHITE {
        key ^= POLY_KEYS[780];
    }
    key
}

fn convert_poly_move(poly_move: u16, board: &Board) -> u32 {
    let poly = poly_move as u32;
    let ff = ((poly >> 6) & 7) as usize;
    let fr = ((poly >> 9) & 7) as usize;
    let tf = (poly & 7) as usize;
    let tr = ((poly >> 3) & 7) as usize;
    let pp = (poly >> 12) & 7;
    let mut text = format!(
        "{}{}{}{}",
        FILE_CHAR[ff] as char, RANK_CHAR[fr] as char, FILE_CHAR[tf] as char, RANK_CHAR[tr] as char
    );
    if pp != 0 {
        let prom = match pp {
            1 => 'n',
            2 => 'b',
            3 => 'r',
            _ => 'q',
        };
        text.push(prom);
    }
    parse_move(&text, board)
}
