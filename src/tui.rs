use std::io;

pub fn run() {
    let mut board = crate::board::Board::new();
    let mut input;
    loop {
        board.print();

        input = String::from("");
        println!("Enter move:");
        let result = io::stdin().read_line(&mut input);
        input = input.trim().to_string();
        if result.is_err() || input.len() < 2 || input.len() > 3 {
            println!("invalid input");
            continue;
        }

        let mut chars = input.chars();
        let col = chars.next().unwrap();
        let mut row = chars.next().unwrap() as u8;
        if col < 'a' || col > 'i' || row < 49 || row > 57 {
            println!("invalid coords");
            continue;
        }
        row -= 48;

        let mut wall_place = false;
        let mut vertical_wall = false;
        if input.len() == 3 {
            match chars.next().unwrap() {
                'v' => {
                    vertical_wall = true;
                }
                'h' => {
                    vertical_wall = false;
                }
                _ => {
                    println!("invalid wall orientation");
                    continue;
                }
            }
            wall_place = true;
        }

        let sqnum = crate::board::sqnum_for_coord(col, row);
        // TODO find child in board.moves that reflects input
    }
}
