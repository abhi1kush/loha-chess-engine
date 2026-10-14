//! Interactive console. Commands and move entry stay out of search.

use crate::adjudicate::game_over;
use crate::board::Board;
use crate::control::GuiControl;
use crate::defs::*;
use crate::evaluate::{eval_position, ClassicEval};
use crate::input::{recv_line, write_line};
use crate::listener::ConsoleListener;
use crate::makemove::make_move;
use crate::notation::parse_move;
use crate::ports::{OpeningBook, SearchControl, SearchInfo};
use crate::search::search_position;
use crate::view::print_board;

pub struct Console;

impl Console {
    pub fn run<B: OpeningBook>(&mut self, board: &mut Board, info: &mut SearchInfo, book: &mut B) {
        write_line("Welcome to loha In Console Mode!");
        write_line("Type help for commands\n");

        let mut listener = ConsoleListener { post: true };
        let mut control = GuiControl;
        let mut depth = MAX_DEPTH;
        let mut movetime = 3000i64;
        let mut engine_side = BLACK;
        board.parse_fen(START_FEN);

        loop {
            if board.side == engine_side && !game_over(board) {
                info.start_time = control.now_ms();
                info.depth = depth;
                if movetime != 0 {
                    info.time_set = true;
                    info.stop_time = info.start_time.saturating_add(movetime as u64);
                } else {
                    info.time_set = false;
                }
                search_position(board, info, &ClassicEval, book, &mut listener, &mut control);
                if info.quit {
                    break;
                }
            }

            print!("\nloha > ");
            let _ = std::io::Write::flush(&mut std::io::stdout());
            let Some(raw) = recv_line() else {
                break;
            };
            let raw = raw.trim();
            if raw.is_empty() {
                continue;
            }
            let mut parts = raw.split_whitespace();
            let command = parts.next().unwrap_or("");
            match command {
                "help" => {
                    write_line("Commands:");
                    write_line("quit - quit game");
                    write_line("force - computer will not think");
                    write_line("print - show board");
                    write_line("post - show thinking");
                    write_line("nopost - do not show thinking");
                    write_line("new - start new game");
                    write_line("setboard - set position using fen string");
                    write_line("go - set computer thinking");
                    write_line("depth x - set depth to x");
                    write_line(
                        "time x - set thinking time to x seconds (depth still applies if set)",
                    );
                    write_line("view - show current depth and movetime settings");
                    write_line("enter moves using b7b8q notation");
                }
                "mirror" => {
                    print_board(board);
                    write_line(&format!("Eval {}", eval_position(board)));
                    board.mirror();
                    print_board(board);
                    write_line(&format!("Eval : {}", eval_position(board)));
                    board.mirror();
                }
                "quit" => {
                    info.quit = true;
                    break;
                }
                "post" => listener.post = true,
                "print" => print_board(board),
                "nopost" => listener.post = false,
                "force" => engine_side = BOTH,
                "view" => {
                    if depth == MAX_DEPTH {
                        print!("depth not set ");
                    } else {
                        print!("depth {depth}");
                    }
                    if movetime != 0 {
                        println!(" movetime {}s", movetime / 1000);
                    } else {
                        println!(" movetime not set");
                    }
                }
                "depth" => {
                    depth = parts.next().and_then(|v| v.parse().ok()).unwrap_or(depth);
                    if depth == 0 {
                        depth = MAX_DEPTH;
                    }
                }
                "time" => {
                    movetime = parts
                        .next()
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or(movetime / 1000)
                        * 1000;
                }
                "new" => {
                    board.hash_table.clear();
                    engine_side = BLACK;
                    board.parse_fen(START_FEN);
                }
                "setboard" => {
                    engine_side = BOTH;
                    let fen = raw.strip_prefix("setboard").unwrap_or("").trim();
                    board.parse_fen(fen);
                }
                "go" => engine_side = board.side,
                _ => {
                    let mov = parse_move(raw, board);
                    if mov == NOMOVE {
                        write_line(&format!("Command unknown:{raw}"));
                        continue;
                    }
                    make_move(board, mov);
                    board.ply = 0;
                }
            }
        }
    }
}
