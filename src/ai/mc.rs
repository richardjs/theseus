use crate::Board;

use rand::seq::SliceRandom;

const ITERATIONS: u32 = 100;

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
            let mut winner = None;
            while winner.is_none() {
                if current.remaining_walls()[0] == 0 && current.remaining_walls()[1] == 0 {
                    if current.shortest_path(current.turn()).len()
                        <= current.shortest_path(current.turn().other()).len()
                    {
                        winner = Some(current.turn());
                    } else {
                        winner = Some(current.turn().other());
                    }
                    break;
                }

                let moves = current.moves_detailed(false, true);
                let mut next = moves.choose(&mut rng).unwrap();
                while !next.paths_exist() {
                    next = moves.choose(&mut rng).unwrap();
                }
                current = next.clone();
                winner = current.winner();
            }

            if winner.unwrap() == board.turn() {
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
