//! Iterative deepening, alpha-beta, and quiescence.
//!
//! The algorithm asks an evaluator for scores, a book for an opening move, a
//! listener for output, and a control for the clock. It does not know which
//! protocol or evaluation is plugged in.

use crate::attack::sq_attacked;
use crate::board::Board;
use crate::defs::*;
use crate::makemove::{make_move, make_null_move, take_move, take_null_move};
use crate::movegen::{generate_all_caps, generate_all_moves, move_exists};
use crate::ports::{Evaluator, OpeningBook, SearchControl, SearchInfo, SearchListener};

fn checkup<C: SearchControl>(info: &mut SearchInfo, control: &mut C) {
    if info.time_set && control.now_ms() > info.stop_time {
        info.stopped = true;
    }
    control.poll(info);
}

fn pick_next_move(list: &mut MoveList, movenum: usize) {
    let mut best_score = 0;
    let mut best_num = movenum;
    for i in movenum..list.count {
        if list.moves[i].score > best_score {
            best_score = list.moves[i].score;
            best_num = i;
        }
    }
    list.moves.swap(movenum, best_num);
}

fn is_repetition(board: &Board) -> bool {
    let mut i = (board.history_ply - board.fifty_move).max(0);
    while i < board.history_ply - 1 {
        if board.pos_key == board.history[i as usize].pos_key {
            return true;
        }
        i += 1;
    }
    false
}

fn quiescence<E: Evaluator, C: SearchControl>(
    mut alpha: i32,
    beta: i32,
    board: &mut Board,
    info: &mut SearchInfo,
    evaluator: &E,
    control: &mut C,
) -> i32 {
    if info.nodes & 2047 == 0 {
        checkup(info, control);
    }
    info.nodes += 1;

    if is_repetition(board) || board.fifty_move >= 100 {
        return 0;
    }
    if board.ply > MAX_DEPTH - 1 {
        return evaluator.evaluate(board);
    }

    let mut score = evaluator.evaluate(board);
    if score >= beta {
        return beta;
    }
    if score > alpha {
        alpha = score;
    }

    let mut list = MoveList::default();
    generate_all_caps(board, &mut list);

    for movenum in 0..list.count {
        pick_next_move(&mut list, movenum);
        if !make_move(board, list.moves[movenum].mov) {
            continue;
        }
        score = -quiescence(-beta, -alpha, board, info, evaluator, control);
        take_move(board);
        if info.stopped {
            return 0;
        }
        if score > alpha {
            if score >= beta {
                return beta;
            }
            alpha = score;
        }
    }
    alpha
}

fn clear_for_search(board: &mut Board, info: &mut SearchInfo) {
    board.search_history = [[0; SQ_NUM]; 13];
    board.search_killers = [[NOMOVE; MAX_DEPTH as usize]; 2];
    board.hash_table.overwrite = 0;
    board.hash_table.hit = 0;
    board.hash_table.cut = 0;
    board.ply = 0;
    info.stopped = false;
    info.nodes = 0;
}

fn alpha_beta<E: Evaluator, C: SearchControl>(
    mut alpha: i32,
    beta: i32,
    mut depth: i32,
    board: &mut Board,
    info: &mut SearchInfo,
    evaluator: &E,
    control: &mut C,
    do_null: bool,
) -> i32 {
    if depth <= 0 {
        return quiescence(alpha, beta, board, info, evaluator, control);
    }
    if info.nodes & 2047 == 0 {
        checkup(info, control);
    }
    info.nodes += 1;

    if (is_repetition(board) || board.fifty_move >= 100) && board.ply != 0 {
        return 0;
    }
    if board.ply > MAX_DEPTH - 1 {
        return evaluator.evaluate(board);
    }

    let in_check = sq_attacked(board, board.king_sq[board.side], board.side ^ 1);
    if in_check {
        depth += 1;
    }

    let (cutoff, pv_move, hash_score) =
        board
            .hash_table
            .probe(board.pos_key, board.ply, alpha, beta, depth);
    if cutoff {
        board.hash_table.cut += 1;
        return hash_score;
    }

    if do_null && !in_check && board.ply != 0 && board.big_piece[board.side] > 0 && depth >= 4 {
        make_null_move(board);
        let null_score = -alpha_beta(
            -beta,
            -beta + 1,
            depth - 4,
            board,
            info,
            evaluator,
            control,
            false,
        );
        take_null_move(board);
        if info.stopped {
            return 0;
        }
        if null_score >= beta && null_score.abs() < IS_MATE {
            return beta;
        }
    }

    let mut list = MoveList::default();
    generate_all_moves(board, &mut list);

    let old_alpha = alpha;
    let mut best_move = NOMOVE;
    let mut best_score = -INFINITE;
    let mut legal = 0;

    if pv_move != NOMOVE {
        for entry in list.moves.iter_mut().take(list.count) {
            if entry.mov == pv_move {
                entry.score = 2_000_000;
                break;
            }
        }
    }

    for movenum in 0..list.count {
        pick_next_move(&mut list, movenum);
        let mov = list.moves[movenum].mov;
        if !make_move(board, mov) {
            continue;
        }
        legal += 1;
        let score = -alpha_beta(
            -beta,
            -alpha,
            depth - 1,
            board,
            info,
            evaluator,
            control,
            true,
        );
        take_move(board);

        if info.stopped {
            return 0;
        }
        if score > best_score {
            best_score = score;
            best_move = mov;
            if score > alpha {
                if score >= beta {
                    if (mov & MFLAG_CAP) == 0 {
                        let ply = board.ply as usize;
                        board.search_killers[1][ply] = board.search_killers[0][ply];
                        board.search_killers[0][ply] = mov;
                    }
                    board.hash_table.store(
                        board.pos_key,
                        beta,
                        best_move,
                        HF_BETA,
                        depth,
                        board.ply,
                    );
                    return beta;
                }
                alpha = score;
                best_move = mov;
                if (mov & MFLAG_CAP) == 0 {
                    let piece = board.pieces[from_sq(best_move) as usize] as usize;
                    board.search_history[piece][to_sq(best_move) as usize] += depth;
                }
            }
        }
    }

    if legal == 0 {
        if in_check {
            return -MATE_SCORE + board.ply;
        }
        return 0;
    }

    if alpha != old_alpha {
        board.hash_table.store(
            board.pos_key,
            best_score,
            best_move,
            HF_EXACT,
            depth,
            board.ply,
        );
    } else {
        board
            .hash_table
            .store(board.pos_key, alpha, best_move, HF_ALPHA, depth, board.ply);
    }
    alpha
}

fn pv_line(board: &mut Board, depth: i32) -> i32 {
    let mut mov = board.hash_table.probe_pv(board.pos_key);
    let mut count = 0i32;
    while mov != NOMOVE && count < depth {
        if move_exists(board, mov) {
            make_move(board, mov);
            board.pv_array[count as usize] = mov;
            count += 1;
        } else {
            break;
        }
        mov = board.hash_table.probe_pv(board.pos_key);
    }
    while board.ply > 0 {
        take_move(board);
    }
    count
}

pub fn search_position<E, B, L, C>(
    board: &mut Board,
    info: &mut SearchInfo,
    evaluator: &E,
    book: &mut B,
    listener: &mut L,
    control: &mut C,
) -> u32
where
    E: Evaluator,
    B: OpeningBook + ?Sized,
    L: SearchListener,
    C: SearchControl,
{
    clear_for_search(board, info);

    let mut best_move = book.suggest(board);

    if best_move == NOMOVE {
        for current_depth in 1..=info.depth {
            let best_score = alpha_beta(
                -INFINITE,
                INFINITE,
                current_depth,
                board,
                info,
                evaluator,
                control,
                true,
            );
            if info.stopped {
                break;
            }
            let pv_moves = pv_line(board, current_depth);
            best_move = board.pv_array[0];
            let elapsed = control.now_ms().saturating_sub(info.start_time);
            listener.on_iteration(
                board,
                info.nodes,
                best_score,
                current_depth,
                elapsed,
                pv_moves,
            );
        }
    }

    listener.on_best_move(board, best_move);
    best_move
}
