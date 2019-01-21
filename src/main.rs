extern crate theseus;

use rand::prelude::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: theseus [tqbn]");
        std::process::exit(1);
    }
    let tqbn = &args[1].to_string();
    let board = theseus::Board::from_tqbn(tqbn);
    board.print();
    let moves = board.moves();
    let child = &moves[rand::thread_rng().gen_range(0, moves.len())];
    child.print();
    let move_string = board.move_string_to(child);
    eprintln!("{}", move_string);
    println!("{}", move_string);
}
