use crate::Board;

const DEPTH: u8 = 2;

fn search(board: &Board, depth: u8) -> i32 {
    if depth == DEPTH {
        if let Some(winner) = board.winner() {
            if winner == board.turn() {
                return std::i32::MAX;
            } else {
                return -std::i32::MAX;
            }
        }
        if board.remaining_walls()[0] == 0 && board.remaining_walls()[1] == 0 {
            let winner;
            if board.shortest_path(board.turn()).len()
                <= board.shortest_path(board.turn().other()).len()
            {
                winner = Some(board.turn());
            } else {
                winner = Some(board.turn().other());
            }
            if winner.unwrap() == board.turn() {
                return std::i32::MAX;
            } else {
                return -std::i32::MAX;
            }
        }
        return 2
            * (board.remaining_walls()[board.turn() as usize]
                - board.remaining_walls()[board.turn().other() as usize]) as i32
            + 1 * (board.shortest_path(board.turn().other()).len() as i32
                - board.shortest_path(board.turn()).len() as i32);
    }
    let mut best_score = -std::i32::MAX;
    for child in board.moves() {
        let score = -search(&child, depth + 1);
        if score >= best_score {
            best_score = score;
        }
    }
    best_score
}

pub fn minimax(board: &Board) -> Board {
    eprintln!("minimax");
    let moves = board.moves_detailed(true, true);
    if moves.len() == 1 {
        return moves[0].clone();
    }

    let mut best_score = -std::i32::MAX;
    let mut best_child = board.moves()[0].clone();
    for child in board.moves() {
        let mut score = -search(&child, 1);
        if board.pawns()[board.turn() as usize] != board.pawns()[board.turn() as usize] {
            score += 1;
        }
        if score >= best_score {
            best_score = score;
            best_child = child;
        }
    }
    eprintln!("score: {}", best_score);
    best_child.clone()
}
