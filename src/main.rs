extern crate theseus;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: theseus [tqbn]");
        std::process::exit(1);
    }
    let tqbn = &args[1].to_string();
    let board = theseus::Board::from_tqbn(tqbn);
    board.print();
    let child = &board.moves()[0];
    child.print();
    println!("{}", board.move_string_to(child));
}
