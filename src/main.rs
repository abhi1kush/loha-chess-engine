//! Composition root. This is the one place that chooses concrete protocols and the book.

use std::env;
use std::time::Instant;

use loha::board::Board;
use loha::console::Console;
use loha::input::{self, write_line};
use loha::perft::{perft_divide, run_suite};
use loha::polybook::PolyglotBook;
use loha::ports::{BookSwitch, SearchInfo};
use loha::uci::Uci;
use loha::xboard::Xboard;
use loha::NAME;
use loha::START_FEN;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Some(pos) = args.iter().position(|arg| arg == "perft") {
        run_perft_cli(&args[pos + 1..]);
        return;
    }
    if let Some(pos) = args.iter().position(|arg| arg == "suite") {
        run_suite_cli(&args[pos + 1..]);
        return;
    }

    let mut book = PolyglotBook::load();
    if args.iter().any(|arg| arg.starts_with("NoBook")) {
        book.set_enabled(false);
        println!("Book Off");
    }

    let mut board = Board::new();
    board.init_hash(64);
    let mut info = SearchInfo::default();
    input::start_input();

    write_line(&format!(
        "Welcome to {NAME}! type cons for console mode ..."
    ));

    loop {
        let Some(line) = input::recv_line() else {
            break;
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("uci") {
            Uci.run(&mut board, &mut info, &mut book);
            if info.quit {
                break;
            }
        } else if line.starts_with("xboard") {
            Xboard.run(&mut board, &mut info, &mut book);
            if info.quit {
                break;
            }
        } else if line.starts_with("cons") {
            Console.run(&mut board, &mut info, &mut book);
            if info.quit {
                break;
            }
        } else if line.starts_with("quit") {
            break;
        }
    }
}

fn run_perft_cli(args: &[String]) {
    let depth: i32 = args.first().and_then(|v| v.parse().ok()).unwrap_or(5);
    let fen = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        START_FEN.to_string()
    };
    let mut board = Board::new();
    if !board.parse_fen(&fen) {
        eprintln!("FEN error");
        std::process::exit(1);
    }
    print!("{board}");
    println!("\nStarting Test To Depth : {depth}");
    let start = Instant::now();
    let nodes = perft_divide(&mut board, depth);
    let ms = start.elapsed().as_millis();
    println!("\n Test Complete : {nodes} nodes visited in {ms}ms");
}

fn run_suite_cli(args: &[String]) {
    let depth: i32 = args.first().and_then(|v| v.parse().ok()).unwrap_or(3);
    let path = "perftsuite.epd";
    match run_suite(path, depth) {
        Ok(failures) if failures.is_empty() => {
            println!("perft suite passed through depth {depth}");
        }
        Ok(failures) => {
            for fail in &failures {
                println!(
                    "FAIL d{} expected {} got {} {}",
                    fail.depth, fail.expected, fail.got, fail.fen
                );
            }
            println!("{} failure(s)", failures.len());
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
