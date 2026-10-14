//! Text diagram of a position.

use std::fmt;

use crate::board::Board;
use crate::defs::*;

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n Game Board : \n")?;
        for rank in (RANK_1..=RANK_8).rev() {
            write!(f, "{} ", rank + 1)?;
            for file in FILE_A..=FILE_H {
                let sq = fr2sq(file, rank);
                let piece = self.pieces[sq as usize];
                let ch = if piece == OFFBOARD as u8 || piece as usize >= PIECE_CHAR.len() {
                    '?'
                } else {
                    PIECE_CHAR[piece as usize] as char
                };
                write!(f, "{ch:>3}")?;
            }
            writeln!(f)?;
        }
        write!(f, "\n  ")?;
        for file in FILE_A..=FILE_H {
            write!(f, "{:>3}", (b'a' + file as u8) as char)?;
        }
        writeln!(f)?;
        let side = SIDE_CHAR.get(self.side).copied().unwrap_or(b'?') as char;
        writeln!(f, "side: {side}")?;
        writeln!(f, "enpass : {} ", self.en_passant)?;
        let castle = format!(
            "{}{}{}{}",
            if (self.castle_perm & WKCA) != 0 {
                'K'
            } else {
                '-'
            },
            if (self.castle_perm & WQCA) != 0 {
                'Q'
            } else {
                '-'
            },
            if (self.castle_perm & BKCA) != 0 {
                'k'
            } else {
                '-'
            },
            if (self.castle_perm & BQCA) != 0 {
                'q'
            } else {
                '-'
            },
        );
        writeln!(f, "castle :{castle}")?;
        writeln!(f, "poskey {:X}", self.pos_key)?;
        Ok(())
    }
}

pub fn print_board(board: &Board) {
    print!("{board}");
}
