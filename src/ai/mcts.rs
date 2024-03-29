use std::cell::RefCell;
use std::f64::INFINITY;
use std::rc::Rc;

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;

use crate::board::Board;

const ITERATIONS: u32 = 50000;
const UCTC: f64 = 10000.0;

const UCTW: f64 = 0.0;
const MOVE_PROBABILITY: f64 = 0.8;

const SIM_THRESHOLD: u32 = 5;

const SIM_EXTEND_PATH_BIAS: f64 = 0.1;
const SIM_EXTEND_PATH_THRESHOLD: usize = 1;
const SIM_SHORTEST_WALK_BIAS: f64 = 0.5;
const PATH_DIFF_COEFF: f64 = 1.0;

const THREADS: u32 = 2;

struct Node {
    board: Board,
    children: Vec<Rc<RefCell<Node>>>,
    value: f64,
    visits: u32,
}

impl Node {
    fn new(board: Board) -> Node {
        Node {
            board,
            children: Vec::new(),
            value: 0.0,
            visits: 0,
        }
    }

    fn expand(&mut self) {
        for child in self.board.moves_detailed(false, true, true, true) {
            self.children.push(Rc::new(RefCell::new(Node::new(child))));
        }
    }

    fn update(&mut self, value: f64) {
        self.visits += 1;
        self.value = (self.value * (self.visits - 1) as f64 + value) / self.visits as f64;
    }
}

fn simulate(mut board: Board) -> f64 {
    let mut rng = thread_rng();
    let turn = board.turn();

    'turn: while !board.can_win() {
        // early termination with no walls remaining
        if board.remaining_walls()[0] == 0 && board.remaining_walls()[1] == 0 {
            return board.shortest_path(turn).len() as f64
                - board.shortest_path(turn.other()).len() as f64;
        }

        /*
        if board.remaining_walls()[board.turn().other() as usize] == 0
            && board.shortest_path(board.turn()).len()
                <= board.shortest_path(board.turn().other()).len()
        {
            return board.shortest_path(turn).len() as f64
                - board.shortest_path(turn.other()).len() as f64;
        }
        */

        if board.remaining_walls()[board.turn() as usize] > 0 && rng.gen_bool(SIM_EXTEND_PATH_BIAS)
        {
            let shortest_path = board.shortest_path(board.turn().other()).len();
            for child in board.moves_detailed(false, true, true, false) {
                if child.shortest_path(child.turn()).len()
                    > shortest_path + SIM_EXTEND_PATH_THRESHOLD
                {
                    board = child.clone();
                    continue 'turn;
                }
            }
        }

        // bias towards walking along shortest path
        if rng.gen_bool(SIM_SHORTEST_WALK_BIAS) {
            for child in board.moves_detailed(true, false, false, true) {
                if child.other_pawn() == *board.shortest_path(board.turn()).first().unwrap()
                //&& !child.can_win()
                {
                    board = child.clone();
                    continue 'turn;
                }
            }
        }

        let moves = board.moves_detailed(false, false, true, true);
        if moves.len() == 1 {
            board = moves[0].clone();
            continue;
        }

        let mut choices: Vec<_> = (0..moves.len()).collect();
        let mut next;
        loop {
            let choice = *choices.choose(&mut rng).unwrap();
            next = &moves[choice];

            let index = choices.iter().position(|x| *x == choice).unwrap();
            choices.remove(index);

            if choices.len() == 0 || next.paths_exist() {
                //&& !next.can_win()) {
                break;
            };
        }

        while !next.paths_exist() {
            next = moves.choose(&mut rng).unwrap();
        }

        board = next.clone();
    }

    return board.shortest_path(turn).len() as f64 - board.shortest_path(turn.other()).len() as f64;
}

fn solver(node: &Rc<RefCell<Node>>) -> f64 {
    let mut node = node.borrow_mut();

    if node.children.len() == 0 {
        node.expand();
    }

    if node.board.can_win() {
        node.update(INFINITY);
        return INFINITY;
    }
    if node.board.remaining_walls()[0] == 0 && node.board.remaining_walls()[1] == 1 {
        if node.board.shortest_path(node.board.turn()).len()
            <= node.board.shortest_path(node.board.turn().other()).len()
        {
            node.update(INFINITY);
            return INFINITY;
        } else {
            node.update(-INFINITY);
            return -INFINITY;
        }
    }

    let mut selected = &node.children[0];
    let mut best_uct = -INFINITY;
    for child in &node.children {
        let c = child.borrow();
        if c.visits == 0 {
            selected = child;
            break;
        }

        let probability = if node.board.pawns()[node.board.turn() as usize]
            == c.board.pawns()[node.board.turn() as usize]
        {
            1.0 - MOVE_PROBABILITY
        } else {
            MOVE_PROBABILITY
        };
        let uct = -c.value
            + (UCTC * (node.visits as f64).ln() / c.visits as f64).sqrt()
            + (UCTW * probability / (c.visits + 1) as f64);
        if uct > best_uct {
            selected = child;
            best_uct = uct;
        }
    }

    let mut r;
    if selected.borrow().value == INFINITY || selected.borrow().value == -INFINITY {
        r = -selected.borrow().value;
    } else {
        if selected.borrow().visits < SIM_THRESHOLD {
            let path_difference = simulate(selected.borrow().board.clone());
            r = if path_difference >= 0.0 { 1.0 } else { -1.0 };
            r += path_difference * PATH_DIFF_COEFF;
            selected.borrow_mut().update(-r);
        } else {
            r = -solver(&Rc::clone(selected));
        }
    }

    if r == -INFINITY {
        for child in &node.children {
            if child.borrow().value != INFINITY {
                // TODO this might need to be adjusted if PATH_DIFF_COEFF ends up being useful (since |r| can be > 1.0)
                r = -1.0;
                break;
            }
        }
    }

    node.update(r);
    r
}

pub fn mcts(board: &Board, log: &mut String) -> Board {
    log.push_str(&format!("theseus {}\n", env!("CARGO_PKG_VERSION")));
    log.push_str(&format!("commit\t{}\n", env!("HEAD_SHA")));
    log.push_str(&format!("patch\t{}\n\n", env!("PATCH_SHA")));

    log.push_str("mcts-solver search\n");
    log.push_str(&format!("iterations:\t{}\n", ITERATIONS));
    log.push_str(&format!("threads:\t{}\n\n", THREADS));
    //log.push_str(&format!("total:\t\t{}\n\n", ITERATIONS * THREADS));

    let (results_tx, results_rx) = mpsc::channel();

    let start_time = SystemTime::now();

    for _ in 0..THREADS {
        let board = board.clone();
        let results_tx = results_tx.clone();
        thread::spawn(move || {
            let root = Rc::new(RefCell::new(Node::new(board.clone())));
            for _ in 0..ITERATIONS {
                solver(&root);
            }

            let mut results = Vec::new();
            for child in &root.borrow().children {
                results.push((child.borrow().value, child.borrow().visits));
            }
            results_tx.send(results).unwrap();
        });
    }
    drop(results_tx);

    let root = Rc::new(RefCell::new(Node::new(board.clone())));
    root.borrow_mut().expand();
    for data in results_rx {
        for (i, (value, visits)) in data.iter().enumerate() {
            root.borrow_mut().children[i].borrow_mut().value += value;
            root.borrow_mut().children[i].borrow_mut().visits += visits;
        }
    }

    let end_time = SystemTime::now();

    let mut best_score = -INFINITY;
    let mut best_child = root.borrow().children[0].clone();
    let mut walking_shortest_path = false;
    for child in &root.borrow().children {
        if -child.borrow().value > best_score {
            best_score = -child.borrow().value;
            best_child = child.clone();
            walking_shortest_path = false;
        } else if -child.borrow().value == best_score {
            // prioritizing walking shortest path
            let board = board.clone();
            if child.borrow().board.other_pawn()
                == *board.shortest_path(board.turn()).first().unwrap()
            {
                best_child = child.clone();
                walking_shortest_path = true;
            }
        }
    }

    let think_time = end_time.duration_since(start_time);
    if think_time.is_ok() {
        let millis = think_time.unwrap().as_millis();
        log.push_str(&format!("time:\t\t{} ms\n", millis));
        log.push_str(&format!(
            "iter/s:\t\t{:.3}\n",
            (ITERATIONS) as f64 / (millis as f64 / 1000.0)
        ));
    }
    log.push_str(&format!("moves:\t\t{}\n\n", root.borrow().children.len()));

    if walking_shortest_path {
        log.push_str(&format!("walking shortest path\n"));
    }
    log.push_str(&format!(
        "value:\t\t{:.3}\n",
        -best_child.borrow().value / (THREADS as f64)
    ));
    log.push_str(&format!("visits:\t\t{}\n", best_child.borrow().visits));
    log.push_str(&format!(
        "focus:\t\t{:.3}\n",
        (best_child.borrow().visits as f64)
            / ((ITERATIONS * THREADS) as f64 / root.borrow().children.len() as f64)
    ));
    log.push_str(&format!(
        "visit %:\t{:.3}%\n\n",
        100.0 * best_child.borrow().visits as f64 / (ITERATIONS * THREADS) as f64
    ));

    let board = best_child.borrow().board.clone();
    board
}
