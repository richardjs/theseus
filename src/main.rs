extern crate theseus;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: theseus [tqbn]");
        std::process::exit(1);
    }

    let tqbn = &args[1].to_string().to_ascii_lowercase();

    eprintln!("input: {}", tqbn);
    let board = theseus::Board::from_tqbn(tqbn);
    board.print();

    let child = theseus::ai::mc(&board);
    let move_string = board.move_string_to(&child);
    eprintln!("output: {}", move_string);
    child.print();

    println!("{}", move_string);
}
