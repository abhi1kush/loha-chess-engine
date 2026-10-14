//! XBoard protocol.

use crate::adjudicate::game_over;
use crate::board::Board;
use crate::control::GuiControl;
use crate::defs::*;
use crate::evaluate::ClassicEval;
use crate::input::{recv_line, write_line};
use crate::listener::XboardListener;
use crate::makemove::make_move;
use crate::notation::parse_move;
use crate::ports::{OpeningBook, SearchControl, SearchInfo};
use crate::search::search_position;

pub struct Xboard;

impl Xboard {
    pub fn run<B: OpeningBook>(&mut self, board: &mut Board, info: &mut SearchInfo, book: &mut B) {
        let mut listener = XboardListener;
        let mut control = GuiControl;
        print_options();

        let mut depth = -1i32;
        let mut movestogo = [30i32, 30];
        let mut time = -1i64;
        let mut inc = 0i64;
        let mut engine_side = BLACK;
        let mut mps = 0i32;

        board.parse_fen(START_FEN);

        loop {
            if board.side == engine_side && !game_over(board) {
                info.start_time = control.now_ms();
                info.depth = depth;
                if time != -1 {
                    info.time_set = true;
                    let slice = time / movestogo[board.side].max(1) as i64;
                    info.stop_time = (info.start_time as i64 + slice - 50 + inc).max(0) as u64;
                } else {
                    info.time_set = false;
                }
                if depth == -1 || depth > MAX_DEPTH {
                    info.depth = MAX_DEPTH;
                }
                write_line(&format!(
                    "time:{time:5} start:{:5} stop:{:5} depth:{:5} timeset:{:5} movestogo:{:5} mps:{mps:5}",
                    info.start_time,
                    info.stop_time,
                    info.depth,
                    info.time_set as u8,
                    movestogo[board.side]
                ));
                search_position(board, info, &ClassicEval, book, &mut listener, &mut control);
                if info.quit {
                    break;
                }
                if mps != 0 {
                    let other = board.side ^ 1;
                    movestogo[other] -= 1;
                    if movestogo[other] < 1 {
                        movestogo[other] = mps;
                    }
                }
            }

            let Some(raw) = recv_line() else {
                break;
            };
            let raw = raw.trim();
            if raw.is_empty() {
                continue;
            }
            write_line(&format!("command seen:{raw}"));
            let mut parts = raw.split_whitespace();
            let command = parts.next().unwrap_or("");

            match command {
                "quit" => {
                    info.quit = true;
                    break;
                }
                "force" => engine_side = BOTH,
                "protover" => print_options(),
                "sd" => {
                    if let Some(value) = parts.next() {
                        depth = value.parse().unwrap_or(depth);
                    }
                    write_line(&format!("DEBUG depth:{depth}"));
                }
                "st" => {
                    if let Some(value) = parts.next() {
                        let movetime: i64 = value.parse().unwrap_or(0);
                        time = movetime * 1000;
                    }
                    write_line(&format!("DEBUG movetime:{time}"));
                }
                "time" => {
                    if let Some(value) = parts.next() {
                        time = value.parse::<i64>().unwrap_or(0) * 10;
                    }
                    write_line(&format!("DEBUG time:{time}"));
                }
                "memory" => {
                    let mb = parts
                        .next()
                        .and_then(|v| v.parse::<usize>().ok())
                        .unwrap_or(16)
                        .clamp(4, MAX_HASH_MB);
                    write_line(&format!("Set Hash to {mb} MB"));
                    board.init_hash(mb);
                }
                "level" => {
                    let nums: Vec<&str> = parts.collect();
                    if let Some((base, extra_sec, inc_s, moves)) = parse_level(&nums) {
                        mps = moves;
                        inc = inc_s;
                        let left = base * 60_000 + extra_sec * 1000;
                        movestogo = [30, 30];
                        if mps != 0 {
                            movestogo = [mps, mps];
                        }
                        time = -1;
                        write_line(&format!(
                            "DEBUG level timeLeft:{left} movesToGo:{} inc:{inc} mps{mps}",
                            movestogo[0]
                        ));
                    }
                }
                "ping" => {
                    let rest = raw.strip_prefix("ping").unwrap_or("");
                    write_line(&format!("pong{rest}"));
                }
                "new" => {
                    board.hash_table.clear();
                    engine_side = BLACK;
                    board.parse_fen(START_FEN);
                    depth = -1;
                    time = -1;
                }
                "setboard" => {
                    engine_side = BOTH;
                    let fen = raw.strip_prefix("setboard").unwrap_or("").trim();
                    board.parse_fen(fen);
                }
                "go" => engine_side = board.side,
                "usermove" => {
                    movestogo[board.side] = movestogo[board.side].saturating_sub(1);
                    let text = raw.strip_prefix("usermove").unwrap_or("").trim();
                    let mov = parse_move(text, board);
                    if mov != NOMOVE {
                        make_move(board, mov);
                        board.ply = 0;
                    }
                }
                _ => {}
            }
        }
    }
}

fn print_options() {
    write_line("feature ping=1 setboard=1 colors=0 usermove=1");
    write_line("feature done=1");
}

fn parse_level(nums: &[&str]) -> Option<(i64, i64, i64, i32)> {
    if nums.len() < 3 {
        return None;
    }
    let mps: i32 = nums[0].parse().ok()?;
    let inc: i64 = nums.last()?.parse().ok()?;
    let base = nums[1];
    if let Some((mins, secs)) = base.split_once(':') {
        let mins: i64 = mins.parse().ok()?;
        let secs: i64 = secs.parse().ok()?;
        Some((mins, secs, inc, mps))
    } else {
        let mins: i64 = base.parse().ok()?;
        Some((mins, 0, inc, mps))
    }
}
