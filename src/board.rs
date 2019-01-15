enum Player {
    White,
    Black,
}

enum Direction {
    North,
    South,
    East,
    West,
}

/// squares are numbered, rows then columns, starting at a1
pub fn sqnum_for_coord(col: char, row: u8) -> u8 {
    (row - 1) * 9 + (col as u8) - 97
    
}

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
	Board{
	    pawns: [sqnum_for_coord('e', 9), sqnum_for_coord('e', 1)],
	    remaining_walls: [10, 10],
	    hwalls: 0,
	    vwalls: 0,
	    turn: Player::White,
	}
    }

    pub fn moves() -> Vec<Board> {
	
    }
}
