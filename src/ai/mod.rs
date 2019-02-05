//mod mc;
//pub use mc::mc;

mod minimax;
pub use minimax::minimax;

mod mcts;
pub use mcts::mcts;

pub fn default(board: &crate::Board, log: &mut String) -> crate::Board {
    mcts(board, log)
}

//mod random;
//pub use random::random;
