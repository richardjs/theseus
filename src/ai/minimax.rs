use crate::board::Player::*;
use crate::board::{Board, Player};

use std::f64::INFINITY;

const DEPTH: u8 = 2;

fn count_win_steps(board: &Board, player: Player) -> u32 {
    let steps = board.walk_paths(player);
    let mut sum = 0;
    if player == White {
        for sqnum in 0..9 {
            sum += steps[sqnum] as u32;
        }
    } else if player == Black {
        for sqnum in 72..81 {
            sum += steps[sqnum] as u32;
        }
    }
    sum
}

fn evaluate(board: &mut Board) -> f64 {
    if let Some(winner) = board.winner() {
        if winner == board.turn() {
            return INFINITY;
        } else {
            return -INFINITY;
        }
    }

    if board.remaining_walls()[0] == 0 && board.remaining_walls()[1] == 0 {
        if board.shortest_path(board.turn()).len()
            <= board.shortest_path(board.turn().other()).len()
        {
            return INFINITY;
        } else {
            return -INFINITY;
        }
    }

    let turn_win_steps = count_win_steps(&board, board.turn());
    let other_win_steps = count_win_steps(&board, board.turn().other());
    let win_step_difference = other_win_steps as f64 - turn_win_steps as f64;

    (10 * (board.remaining_walls()[board.turn() as usize] as i32
        - board.remaining_walls()[board.turn().other() as usize] as i32)) as f64
        + (10
            * (board.shortest_path(board.turn().other()).len() as i32
                - board.shortest_path(board.turn()).len() as i32)) as f64
        + win_step_difference
}

fn search(board: &mut Board, depth: u8) -> f64 {
    if depth == DEPTH {
        return evaluate(board);
    }

    let mut best_score = -INFINITY;
    for child in board.moves() {
        let mut child = child.clone();
        let score = -search(&mut child, depth + 1);
        if score > best_score {
            best_score = score;
        }
    }
    best_score
}

pub fn minimax(board: &Board, log: &mut String) -> Board {
    log.push_str("minimax search\n");
    log.push_str(&format!("depth:\t{}\n", DEPTH));

    let moves = board.moves_detailed(false, true, true);
    if moves.len() == 1 {
        return moves[0].clone();
    }

    let mut best_score = -INFINITY;
    let mut best_child = board.moves()[0].clone();
    for child in board.moves() {
        let mut child = child.clone();
        let score = -search(&mut child, 1);

        if score > best_score {
            best_score = score;
            best_child = child;
        }
    }

    log.push_str(&format!("score:\t{}\n", best_score));
    best_child.clone()
}
