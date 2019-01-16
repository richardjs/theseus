extern crate theseus;

fn main() {
    let board = theseus::Board::new();
    println!("{:?}", board.moves().len());
}
