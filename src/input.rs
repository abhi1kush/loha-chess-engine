//! Stdin session and the engine clock. Protocols read lines; search does not.

use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Instant;

use crate::ports::SearchInfo;

static INPUT: OnceLock<Mutex<Receiver<String>>> = OnceLock::new();
static CLOCK: OnceLock<Instant> = OnceLock::new();

pub fn now_ms() -> u64 {
    CLOCK.get_or_init(Instant::now).elapsed().as_millis() as u64
}

pub fn start_input() {
    let _ = INPUT.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(line) => {
                        if tx.send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Mutex::new(rx)
    });
}

pub fn recv_line() -> Option<String> {
    let rx = INPUT.get()?;
    let rx = rx.lock().ok()?;
    rx.recv().ok()
}

pub fn poll_input(info: &mut SearchInfo) {
    let Some(rx) = INPUT.get() else {
        return;
    };
    let Ok(rx) = rx.lock() else {
        return;
    };
    loop {
        match rx.try_recv() {
            Ok(line) => {
                let line = line.trim();
                if line == "isready" {
                    write_line("readyok");
                    continue;
                }
                info.stopped = true;
                if line.starts_with("quit") {
                    info.quit = true;
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                info.stopped = true;
                info.quit = true;
                break;
            }
        }
    }
}

pub fn write_line(line: &str) {
    println!("{line}");
    let _ = io::stdout().flush();
}
