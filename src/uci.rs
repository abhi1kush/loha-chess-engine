//! UCI protocol. It configures options and asks search to think.

use crate::board::Board;
use crate::control::GuiControl;
use crate::defs::*;
use crate::evaluate::ClassicEval;
use crate::input::{recv_line, write_line};
use crate::listener::UciListener;
use crate::makemove::make_move;
use crate::notation::parse_move;
use crate::ports::{BookSwitch, OpeningBook, SearchInfo};
use crate::search::search_position;
use crate::view::print_board;

pub struct Uci;

impl Uci {
    pub fn run<B>(&mut self, board: &mut Board, info: &mut SearchInfo, book: &mut B)
    where
        B: OpeningBook + BookSwitch,
    {
        let mut listener = UciListener;
        let mut control = GuiControl;
        write_line(&format!("id name {NAME}"));
        write_line(&format!("id author {AUTHOR}"));
        write_line(&format!(
            "option name Hash type spin default 64 min 4 max {MAX_HASH_MB}"
        ));
        write_line("option name Book type check default true");
        write_line("uciok");

        while let Some(line) = recv_line() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with("isready") {
                write_line("readyok");
            } else if line.starts_with("position") {
                parse_position(line, board);
            } else if line.starts_with("ucinewgame") {
                board.hash_table.clear();
                parse_position("position startpos", board);
            } else if line.starts_with("go") {
                parse_go(line, info, board, book, &mut listener, &mut control);
                if info.quit {
                    break;
                }
            } else if line.starts_with("quit") {
                info.quit = true;
                break;
            } else if line.starts_with("uci") {
                write_line(&format!("id name {NAME}"));
                write_line(&format!("id author {AUTHOR}"));
                write_line("uciok");
            } else if line
                .to_ascii_lowercase()
                .starts_with("setoption name hash value")
            {
                let mb = line
                    .split_whitespace()
                    .filter_map(|tok| tok.parse::<usize>().ok())
                    .last()
                    .unwrap_or(64)
                    .clamp(4, MAX_HASH_MB);
                write_line(&format!("Set Hash to {mb} MB"));
                board.init_hash(mb);
            } else if line
                .to_ascii_lowercase()
                .starts_with("setoption name book value")
            {
                book.set_enabled(line.to_ascii_lowercase().contains("true"));
            }
            if info.quit {
                break;
            }
        }
    }
}

fn parse_position(line: &str, board: &mut Board) {
    let rest = line.strip_prefix("position").unwrap_or(line).trim_start();
    if rest.starts_with("startpos") {
        board.parse_fen(START_FEN);
    } else if let Some(fen_at) = rest.strip_prefix("fen") {
        let fen_at = fen_at.trim_start();
        let fen = fen_at.split(" moves").next().unwrap_or(fen_at);
        if !board.parse_fen(fen.trim()) {
            board.parse_fen(START_FEN);
        }
    } else {
        board.parse_fen(START_FEN);
    }

    if let Some(idx) = rest.find("moves") {
        let moves = rest[idx + "moves".len()..].trim_start();
        for token in moves.split_whitespace() {
            let mov = parse_move(token, board);
            if mov == NOMOVE {
                break;
            }
            make_move(board, mov);
            board.ply = 0;
        }
    }
    print_board(board);
}

fn field(line: &str, key: &str) -> Option<i64> {
    let idx = line.find(key)?;
    let rest = line[idx + key.len()..].trim_start();
    let num: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .collect();
    num.parse().ok()
}

fn parse_go<B, L, C>(
    line: &str,
    info: &mut SearchInfo,
    board: &mut Board,
    book: &mut B,
    listener: &mut L,
    control: &mut C,
) where
    B: OpeningBook + ?Sized,
    L: crate::ports::SearchListener,
    C: crate::ports::SearchControl,
{
    let mut depth = -1i32;
    let mut movestogo = 30i64;
    let mut movetime = -1i64;
    let mut time = -1i64;
    let mut inc = 0i64;
    info.time_set = false;

    if board.side == BLACK {
        if let Some(value) = field(line, "binc") {
            inc = value;
        }
    }
    if board.side == WHITE {
        if let Some(value) = field(line, "winc") {
            inc = value;
        }
    }
    if board.side == WHITE {
        if let Some(value) = field(line, "wtime") {
            time = value;
        }
    }
    if board.side == BLACK {
        if let Some(value) = field(line, "btime") {
            time = value;
        }
    }
    if let Some(value) = field(line, "movestogo") {
        movestogo = value.max(1);
    }
    if let Some(value) = field(line, "movetime") {
        movetime = value;
    }
    if let Some(value) = field(line, "depth") {
        depth = value as i32;
    }
    if movetime != -1 {
        time = movetime;
        movestogo = 1;
    }

    info.start_time = control.now_ms();
    info.depth = depth;
    if time != -1 {
        info.time_set = true;
        time /= movestogo;
        time -= 50;
        info.stop_time = (info.start_time as i64).saturating_add(time + inc).max(0) as u64;
    }
    if depth == -1 {
        info.depth = MAX_DEPTH;
    }
    write_line(&format!(
        "time: {time} start :{} stop : {} depth : {} timeset :{}",
        info.start_time, info.stop_time, info.depth, info.time_set as u8
    ));
    search_position(board, info, &ClassicEval, book, listener, control);
}
