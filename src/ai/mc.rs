use crate::Board;

use rand::prelude;
use rand::seq::SliceRandom;

use std::collections::HashMap;

const ITERATIONS: u32 = 5;

pub fn mc(board: &Board) -> Board {
    let mut rng = rand::thread_rng();
    let moves = board.moves_detailed(true, true);
    if moves.len() == 1 {
        return moves[0].clone();
    }

    let mut wins = Vec::new();
    for _ in 0..moves.len() {
        wins.push(0);
    }

    for (i, child) in moves.iter().enumerate() {
        for _ in 0..ITERATIONS {
            let mut current = child.clone();
            while current.winner().is_none() {
                let moves = current.moves_detailed(false, true);
                let mut next = moves.choose(&mut rng).unwrap();
                while !next.paths_exist() {
                    next = moves.choose(&mut rng).unwrap();
                }
                current = next.clone();
            }
            if current.winner().unwrap() == board.turn() {
                wins[i] = wins[i] + 1;
            }
        }
    }

    let mut max_child = 0;
    let mut max_score = 0;
    for (i, count) in wins.iter().enumerate() {
        if *count > max_score {
            max_child = i;
            max_score = *count;
        }
    }
    eprintln!("score: {}", max_score);
    moves[max_child].clone()
}
