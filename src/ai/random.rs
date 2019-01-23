use crate::board::Board;

use rand::prelude::*;

pub fn random(board: &Board) -> Board {
    let moves = board.moves();
    moves[rand::thread_rng().gen_range(0, moves.len())].clone()
}
