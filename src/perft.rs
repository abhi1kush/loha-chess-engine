//! Perft and the EPD perft suite shipped with the original engine.

use crate::board::Board;
use crate::defs::*;
use crate::makemove::{make_move, take_move};
use crate::movegen::generate_all_moves;
use crate::notation::move_string;

pub fn perft(board: &mut Board, depth: i32) -> u64 {
    if depth <= 0 {
        return 1;
    }
    let mut list = MoveList::default();
    generate_all_moves(board, &mut list);
    let mut nodes = 0u64;
    for n in 0..list.count {
        if !make_move(board, list.moves[n].mov) {
            continue;
        }
        nodes += if depth == 1 {
            1
        } else {
            perft(board, depth - 1)
        };
        take_move(board);
    }
    nodes
}

pub fn perft_divide(board: &mut Board, depth: i32) -> u64 {
    if depth <= 0 {
        println!("\n1");
        return 1;
    }
    let mut list = MoveList::default();
    generate_all_moves(board, &mut list);
    let mut nodes = 0u64;
    for n in 0..list.count {
        let mov = list.moves[n].mov;
        if !make_move(board, mov) {
            continue;
        }
        let count = if depth == 1 {
            1
        } else {
            perft(board, depth - 1)
        };
        take_move(board);
        println!("{}: {count}", move_string(mov));
        nodes += count;
    }
    println!("\n{nodes}");
    nodes
}

pub struct SuiteFailure {
    pub fen: String,
    pub depth: i32,
    pub expected: u64,
    pub got: u64,
}

pub fn run_suite(path: &str, max_depth: i32) -> Result<Vec<SuiteFailure>, String> {
    let text = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut failures = Vec::new();
    let mut board = Board::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (fen, rest) = line
            .split_once(';')
            .ok_or_else(|| format!("bad epd: {line}"))?;
        let fen = fen.trim();
        if !board.parse_fen(fen) {
            return Err(format!("fen error: {fen}"));
        }
        for part in rest.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let mut tokens = part.split_whitespace();
            let label = tokens.next().unwrap_or("");
            let expected: u64 = match tokens.next().unwrap_or("").parse() {
                Ok(value) => value,
                Err(_) => continue,
            };
            let depth: i32 = label
                .trim_start_matches(['D', 'd'])
                .parse()
                .map_err(|_| format!("bad depth in {part}"))?;
            if depth > max_depth {
                continue;
            }
            let key = board.pos_key;
            let got = perft(&mut board, depth);
            if board.pos_key != key {
                return Err(format!("position not restored after perft: {fen}"));
            }
            if got != expected {
                failures.push(SuiteFailure {
                    fen: fen.to_string(),
                    depth,
                    expected,
                    got,
                });
            }
        }
    }
    Ok(failures)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::OfflineControl;
    use crate::evaluate::ClassicEval;
    use crate::listener::SilentListener;
    use crate::polybook::EmptyBook;
    use crate::ports::SearchInfo;
    use crate::search::search_position;

    fn pos(fen: &str) -> Board {
        let mut board = Board::new();
        assert!(board.parse_fen(fen), "{fen}");
        assert!(board.check_board(), "{fen}");
        board
    }

    #[test]
    fn parses_side_castle_and_en_passant() {
        let board = pos("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        assert_eq!(board.side, BLACK);
        assert_eq!(board.castle_perm, WKCA | WQCA | BKCA | BQCA);
        assert_eq!(board.en_passant, fr2sq(4, 2));
        assert_eq!(board.fifty_move, 0);
    }

    #[test]
    fn start_position_perft() {
        let mut board = pos(START_FEN);
        assert_eq!(perft(&mut board, 1), 20);
        assert_eq!(perft(&mut board, 2), 400);
        assert_eq!(perft(&mut board, 3), 8902);
        assert_eq!(perft(&mut board, 4), 197_281);
    }

    #[test]
    fn kiwipete_perft() {
        let mut board = pos("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(perft(&mut board, 1), 48);
        assert_eq!(perft(&mut board, 2), 2039);
        assert_eq!(perft(&mut board, 3), 97_862);
    }

    #[test]
    fn promotions_and_castling_perft() {
        let mut board = pos("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(perft(&mut board, 1), 44);
        assert_eq!(perft(&mut board, 2), 1486);
        assert_eq!(perft(&mut board, 3), 62_379);

        let mut board = pos("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        assert_eq!(perft(&mut board, 1), 26);
        assert_eq!(perft(&mut board, 2), 568);
        assert_eq!(perft(&mut board, 3), 13_744);

        let mut board = pos("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft(&mut board, 1), 14);
        assert_eq!(perft(&mut board, 2), 191);
        assert_eq!(perft(&mut board, 3), 2812);
        assert_eq!(perft(&mut board, 4), 43_238);
    }

    #[test]
    fn perft_suite_to_depth_two() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/perftsuite.epd");
        let failures = run_suite(path, 2).expect("suite");
        assert!(
            failures.is_empty(),
            "{} failures, first: {} d{} expected {} got {}",
            failures.len(),
            failures[0].fen,
            failures[0].depth,
            failures[0].expected,
            failures[0].got
        );
    }

    #[test]
    fn search_finds_a_legal_move() {
        let mut board = pos(START_FEN);
        board.init_hash(4);
        let mut info = SearchInfo {
            depth: 3,
            start_time: crate::input::now_ms(),
            ..SearchInfo::default()
        };
        let mov = search_position(
            &mut board,
            &mut info,
            &ClassicEval,
            &mut EmptyBook,
            &mut SilentListener,
            &mut OfflineControl,
        );
        assert_ne!(mov, NOMOVE);
        assert!(crate::movegen::move_exists(&mut board, mov));
    }
}
