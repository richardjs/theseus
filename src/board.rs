#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Player {
    White = 0,
    Black = 1,
}

impl Player {
    pub fn other(&self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Player::White
    }
}

enum Direction {
    North,
    South,
    East,
    West,
}
use Direction::*;

impl Direction {
    fn left(&self) -> Direction {
        match self {
            North => West,
            South => East,
            East => North,
            West => South,
        }
    }

    fn right(&self) -> Direction {
        match self {
            North => East,
            South => West,
            East => South,
            West => North,
        }
    }

	fn move_sqnum(&self, sqnum: u8) -> u8{
		match self {
			North => sqnum - 9,
			South => sqnum + 9,
			East => sqnum + 1,
			West => sqnum - 1,
		}
	}
}

/// squares are numbered, left-to-right, top-to-bottom, starting at a9
pub fn sqnum_for_coord(col: char, row: u8) -> u8 {
    (row - 1) * 9 + (col.to_ascii_lowercase() as u8) - 97
}

pub fn sqnum_for_string(string: &str) -> u8 {
    assert_eq!(string.len(), 2);
    let chars: Vec<_> = string.to_lowercase().chars().collect();
    sqnum_for_coord(chars[0], chars[1] as u8)
}

pub fn string_for_sqnum(sqnum: u8) -> String {
    let row = sqnum / 9;
    let col = sqnum % 9;
    let mut string = String::new();
    string.push((col + 97) as char);
    string.push((row + 49) as char);
    string
}

#[derive(Clone, Debug, Default)]
pub struct Board {
    /// pawn position, in square numbers
    pawns: [u8; 2],

    /// each player's remaining walls
    remaining_walls: [u8; 2],

    /// bitboards for horizontal and vertical walls, specifying the center of the wall
    hwalls: u64,
    vwalls: u64,

    /// next player to move
    turn: Player,
}

impl Board {
    pub fn new() -> Board {
        Board {
            pawns: [sqnum_for_coord('e', 9), sqnum_for_coord('e', 1)],
            remaining_walls: [10, 10],
            hwalls: 0,
            vwalls: 0,
            turn: Player::White,
        }
    }

    pub fn from_tqbn(tqbn: &str) -> Board {
        assert!(tqbn.len() == 73);
        let tqbn: Vec<_> = tqbn.chars().collect();

        let mut hwalls = 0;
        let mut vwalls = 0;
        for (i, c) in tqbn[8..72].into_iter().enumerate() {
            match c.to_ascii_lowercase() {
                'h' => hwalls |= 1 << i,
                'v' => vwalls |= 1 << i,
                'n' => {}
                _ => {
                    panic!();
                }
            }
        }

        Board {
            pawns: [
                sqnum_for_coord(tqbn[0], tqbn[1].to_digit(10).unwrap() as u8),
                sqnum_for_coord(tqbn[2], tqbn[3].to_digit(10).unwrap() as u8),
            ],
            remaining_walls: [
                tqbn[4..6].iter().collect::<String>().parse::<u8>().unwrap(),
                tqbn[6..8].iter().collect::<String>().parse::<u8>().unwrap(),
            ],
            hwalls,
            vwalls,
            turn: match tqbn[72] {
                '1' => Player::White,
                '2' => Player::Black,
                _ => panic!(),
            },
        }
    }

    pub fn turn(&self) -> Player {
        self.turn
    }

    fn is_open(&self, sqnum: u8, direction: &Direction) -> bool {
        let se_wall = (sqnum / 9) * 8 + (sqnum % 9);
        let sw_wall = if se_wall > 0 { se_wall - 1 } else { 0 };
        let nw_wall = if se_wall > 9 { se_wall - 9 } else { 0 };
        let ne_wall = if se_wall > 8 { se_wall - 8 } else { 0 };
        match direction {
            North => {
                sqnum > 8
                    && (sqnum % 9 == 0 || ((1 << nw_wall) & self.hwalls) == 0)
                    && ((sqnum + 1) % 9 == 0 || ((1 << ne_wall) & self.hwalls) == 0)
            }
            South => {
                sqnum < 72
                    && (sqnum % 9 == 0 || ((1 << sw_wall) & self.hwalls) == 0)
                    && ((sqnum + 1) % 9 == 0 || ((1 << se_wall) & self.hwalls) == 0)
            }
            East => {
                (sqnum + 1) % 9 != 0
                    && (sqnum < 9 || ((1 << ne_wall) & self.vwalls) == 0)
                    && (sqnum > 71 || ((1 << se_wall) & self.vwalls) == 0)
            }
            West => {
                (sqnum) % 9 != 0
                    && (sqnum < 9 || ((1 << nw_wall) & self.vwalls) == 0)
                    && (sqnum > 71 || ((1 << sw_wall) & self.vwalls) == 0)
            }
        }
    }

    pub fn moves(&self) -> Vec<Board> {
        let turn = self.turn as usize;
        let other = self.turn.other() as usize;
        let pawn = self.pawns[turn];

        let mut moves = vec![];

        // pawn movements
        // TODO hopping over pawns when wall blocks
		for direction in [North, South, East, West].iter() {
			if !self.is_open(pawn, direction) {
				continue;
			}

            let mut child = self.clone();
            child.pawns[turn] = direction.move_sqnum(pawn);
            child.turn = child.turn.other();

            if child.pawns[turn] == child.pawns[other] {
                if self.is_open(child.pawns[turn], direction) {
                    child.pawns[turn] = direction.move_sqnum(child.pawns[turn]);
                    moves.push(child);
                }
            } else {
                moves.push(child);
            }
		}

        // wall placements
        // we're going to start with a fairly naive algorithm, and optimize later
        // TODO checking for blocked paths
        if self.remaining_walls[turn] == 0 {
            return moves;
        }
        for i in 0..64 {
            let wall_bit = 1 << i;
            if (self.hwalls & wall_bit) > 0 || (self.vwalls & wall_bit) > 0 {
                continue;
            }
            if (i == 0 || ((wall_bit >> 1) & self.hwalls) == 0)
                && (i == 63 || ((wall_bit << 1) & self.hwalls == 0))
            {
                let mut child = self.clone();
                child.hwalls |= wall_bit;
                child.remaining_walls[turn] -= 1;
                child.turn = child.turn.other();
                moves.push(child);
            }
            if (i < 8 || ((wall_bit >> 8) & self.vwalls) == 0)
                && (i > 55 || ((wall_bit << 8) & self.vwalls == 0))
            {
                let mut child = self.clone();
                child.vwalls |= wall_bit;
                child.remaining_walls[turn] -= 1;
                child.turn = child.turn.other();
                moves.push(child);
            }
        }

        moves
    }

    pub fn print(&self) {
        eprintln!("  a   b   c   d   e   f   g   h   i");
        for row in 0..9 {
            eprint!("{} ", row + 1);
            for col in 0..9 {
                let sqnum = row * 9 + col;
                let se_wall = (sqnum / 9) * 8 + (sqnum % 9);
                let ne_wall = if se_wall > 7 { se_wall - 8 } else { 0 };
                if self.pawns[Player::White as usize] == sqnum {
                    eprint!("W");
                } else if self.pawns[Player::Black as usize] == sqnum {
                    eprint!("B");
                } else {
                    eprint!(".");
                }
                if col != 8 {
                    if (sqnum > 8 && (self.vwalls & (1 << ne_wall)) > 0)
                        || (sqnum < 72 && (self.vwalls & (1 << se_wall)) > 0)
                    {
                        eprint!(" # ");
                    } else {
                        eprint!("   ");
                    }
                }
            }
            eprintln!();
            if row < 8 {
                for col in 0..9 {
                    eprint!("  ");
                    let sqnum = row * 9 + col;
                    let se_wall = (sqnum / 9) * 8 + (sqnum % 9);
                    let sw_wall = if se_wall > 1 { se_wall - 1 } else { 0 };
                    if ((sqnum + 1) % 9 != 0 && (self.hwalls & (1 << se_wall)) > 0)
                        || (sqnum % 9 != 0) && (self.hwalls & (1 << sw_wall) > 0)
                    {
                        eprint!("# ");
                    } else {
                        eprint!("  ");
                    }
                }
                eprintln!();
            }
        }
    }

    pub fn move_string_to(&self, child: &Board) -> String {
        let turn = self.turn as usize;
        assert!(
            self.turn != child.turn
                && (self.pawns[turn] != child.pawns[turn]
                    || self.remaining_walls[turn] != child.remaining_walls[turn])
        );
        if self.pawns[turn] != child.pawns[turn] {
            return string_for_sqnum(child.pawns[turn]);
        }
        let wall;
        let horizontal;
        if (child.hwalls & !self.hwalls) > 0 {
            wall = child.hwalls & !self.hwalls;
            horizontal = true;
        } else if (child.vwalls & !self.vwalls) > 0 {
            wall = child.vwalls & !self.vwalls;
            horizontal = false;
        } else {
            panic!("no change in walls");
        }
        let wallnum = if wall > 0 { wall.trailing_zeros() } else { 0 } as u8;
        let sqnum = wallnum + wallnum / 8;
        let mut move_string = string_for_sqnum(sqnum);
        move_string.push(if horizontal { 'h' } else { 'v' });
        move_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn only_pawn_moves(original: &Board, moves: Vec<Board>) -> Vec<Board> {
        let mut pawn_moves = vec![];
        for child in moves {
            if original.pawns[original.turn as usize] != child.pawns[original.turn as usize] {
                pawn_moves.push(child);
            }
        }
        pawn_moves
    }

    #[test]
    fn opening_branching_factor() {
        let board = Board::new();
        assert_eq!(board.moves().len(), 131);
    }

    #[test]
    fn simple_wall_blocking() {
        let mut board = Board::new();
        board.pawns[Player::White as usize] = 12;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 4);
        board.hwalls |= 1 << 2;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 3);
        board.hwalls = 0;
        board.hwalls |= 1 << 3;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 3);
        board.hwalls |= 1 << 10;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.hwalls = 0;
        board.hwalls |= 1 << 11;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 3);
        board.vwalls |= 1 << 2;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.vwalls = 0;
        board.vwalls |= 1 << 10;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.vwalls = 0;
        board.vwalls |= 1 << 3;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.hwalls = 0;
        board.vwalls = 0;
        board.vwalls |= 1 << 11;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 3);
    }

    #[test]
    fn corner_moves() {
        let mut board = Board::new();
        board.pawns[Player::White as usize] = 0;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.pawns[Player::White as usize] = 8;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.pawns[Player::White as usize] = 72;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.pawns[Player::White as usize] = 80;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
    }

    #[test]
    fn load_tqbn() {
        let board = Board::from_tqbn(&String::from(
            "e9e11010nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn1",
        ));
        assert_eq!(board.pawns[0], 76);
        assert_eq!(board.pawns[1], 4);
        assert_eq!(board.remaining_walls[0], 10);
        assert_eq!(board.remaining_walls[1], 10);
        assert_eq!(board.hwalls, 0);
        assert_eq!(board.vwalls, 0);
        assert_eq!(board.turn, Player::White);
    }

    #[test]
    fn vertical_wall_place_bug() {
        let board = Board::from_tqbn(
            "e9e10506nnnnnnnnnnvnnnnnnnhnnnnnnnnnnhnnnnvnvnnvnnnnhnnnhnnnnnnnnnnnhnnn2",
        );
        for child in board.moves() {
            assert_ne!(board.move_string_to(&child), "c1v");
        }
        let board = Board::from_tqbn(
            "e9e10606nnnhvnnnnnnnnnnnnnnnnnnnnnvnvnnnnnnnnnvnnnvnnnnnnnnnvnnnnnnvnnnn1",
        );
        for child in board.moves() {
            assert_ne!(board.move_string_to(&child), "e3v");
        }
    }

    #[test]
    fn simple_pawn_jumping() {
        // jump to the north
        let board = Board::from_tqbn(&String::from(
            "e4e51010nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2",
        ));
        let mut jumped = false;
        for child in only_pawn_moves(&board, board.moves()) {
            if board.move_string_to(&child) == "e3" {
                jumped = true;
                break;
            }
        }
        assert!(jumped);
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 4);

        // jump to the south
        let board = Board::from_tqbn(&String::from(
            "e5e41010nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2",
        ));
        let mut jumped = false;
        for child in only_pawn_moves(&board, board.moves()) {
            if board.move_string_to(&child) == "e6" {
                jumped = true;
                break;
            }
        }
        assert!(jumped);
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 4);

        // jump to the east
        let board = Board::from_tqbn(&String::from(
            "e4d41010nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2",
        ));
        let mut jumped = false;
        for child in only_pawn_moves(&board, board.moves()) {
            child.print();
            if board.move_string_to(&child) == "f4" {
                jumped = true;
                break;
            }
        }
        assert!(jumped);
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 4);

        // jump to the west
        let board = Board::from_tqbn(&String::from(
            "d4e41010nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2",
        ));
        let mut jumped = false;
        for child in only_pawn_moves(&board, board.moves()) {
            child.print();
            if board.move_string_to(&child) == "c4" {
                jumped = true;
                break;
            }
        }
        assert!(jumped);
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 4);
    }
}
