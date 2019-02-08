use std::cell::RefCell;
use std::f64::INFINITY;
use std::rc::Rc;

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

use crate::board::Board;
use crate::board::Player;

const ITERATIONS: u32 = 20000;
const UCTC: f64 = 1000.0;
const UCTW: f64 = 1000.0;
const MOVE_PROBABILITY: f64 = 0.9;
const SIM_THRESHOLD: u32 = 5;
const PATH_MOVE_SIM_PROBABILTY: f64 = 0.95;

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
        for child in self.board.moves_detailed(false, true, true) {
            self.children.push(Rc::new(RefCell::new(Node::new(child))));
        }
    }

    fn update(&mut self, value: f64) {
        self.visits += 1;
        self.value = (self.value * (self.visits - 1) as f64 + value) / self.visits as f64;
    }
}

fn simulate(mut board: Board) -> Player {
    let mut rng = thread_rng();

    'turn: while board.winner().is_none() {
        // early termination with no walls remaining
        if board.remaining_walls()[0] == 0 && board.remaining_walls()[1] == 0 {
            if board.shortest_path(board.turn()).len()
                <= board.shortest_path(board.turn().other()).len()
            {
                return board.turn();
            } else {
                return board.turn().other();
            }
        }

        // bias towards walking along shortest path
        if rng.gen_bool(PATH_MOVE_SIM_PROBABILTY) {
            for child in board.moves_detailed(false, false, true) {
                if child.other_pawn() == *board.shortest_path(board.turn()).first().unwrap()
                    && child.paths_exist()
                {
                    board = child.clone();
                    continue 'turn;
                }
            }
        }

        let moves = board.moves_detailed(false, false, true);
        let mut next = moves.choose(&mut rng).unwrap();
        while !next.paths_exist() {
            next = moves.choose(&mut rng).unwrap();
        }
        board = next.clone();
    }

    return board.winner().unwrap();
}

fn solver(node: &Rc<RefCell<Node>>) -> f64 {
    let mut node = node.borrow_mut();

    if node.children.len() == 0 {
        node.expand();
    }

    if node.children.len() == 1 && node.children[0].borrow().board.winner().is_some() {
        node.update(INFINITY);
        return INFINITY;
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
            r = if simulate(selected.borrow().board.clone()) == node.board.turn() {
                1.0
            } else {
                -1.0
            };
            selected.borrow_mut().update(-r);
        } else {
            r = -solver(&Rc::clone(selected));
        }
    }

    if r == -INFINITY {
        for child in &selected.borrow().children {
            if child.borrow().value != INFINITY {
                r = -1.0;
                break;
            }
        }
    }

    node.update(r);
    r
}

pub fn mcts(board: &Board, log: &mut String) -> Board {
    log.push_str("mcts-solver search\n");
    log.push_str(&format!("iterations:\t{}\n", ITERATIONS));

    let root = Rc::new(RefCell::new(Node::new(board.clone())));
    for _ in 0..ITERATIONS {
        solver(&root);
    }

    let mut best_score = -INFINITY;
    let mut best_child = root.borrow().children[0].clone();
    for child in &root.borrow().children {
        if -child.borrow().value > best_score {
            best_score = -child.borrow().value;
            best_child = child.clone();
        }
    }

    log.push_str(&format!("moves:\t\t{}\n", root.borrow().children.len()));
    log.push_str(&format!("visits:\t\t{}\n", best_child.borrow().visits));
    log.push_str(&format!(
        "visit %:\t{}%\n",
        100.0 * best_child.borrow().visits as f64 / ITERATIONS as f64
    ));
    log.push_str(&format!(
        "focus:\t\t{}\n",
        (best_child.borrow().visits as f64)
            / (ITERATIONS as f64 / root.borrow().children.len() as f64)
    ));
    log.push_str(&format!("value:\t\t{}\n", -best_child.borrow().value));

    let board = best_child.borrow().board.clone();
    board
}
