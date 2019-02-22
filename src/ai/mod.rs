//mod mc;
//pub use mc::mc;

//mod minimax;
//pub use minimax::minimax;

mod mcts;
pub use mcts::mcts;

fn presearch(board: &crate::Board, log: &mut String) -> Option<crate::Board> {
    if board.can_win() {
        log.push_str("presearch: taking win\n");
        return Some(board.moves_detailed(true, false, false, true)[0].clone());
    }
    if board.remaining_walls()[0] == 0 && board.remaining_walls()[1] == 0 {
        log.push_str("presearch: walking shortest path\n");
        for child in board.moves_detailed(true, false, false, true) {
            if child.other_pawn() == *board.shortest_path(board.turn()).first().unwrap() {
                return Some(child.clone());
            }
        }
    }
    None
}

pub fn default(board: &crate::Board, log: &mut String) -> crate::Board {
    if let Some(board) = presearch(board, log) {
        return board;
    }
    mcts(board, log)
}

//mod random;
//pub use random::random;
