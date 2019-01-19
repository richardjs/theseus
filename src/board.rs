#[derive(Clone, Copy, Debug)]
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

/// squares are numbered, rows then columns, starting at a1
pub fn sqnum_for_coord(col: char, row: u8) -> u8 {
    (row - 1) * 9 + (col as u8) - 97
}

#[derive(Clone, Debug)]
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

    pub fn moves(&self) -> Vec<Board> {
        let turn = self.turn as usize;
        let pawn = self.pawns[turn];

        let mut moves = vec![];

        // pawn movements
        // TODO confirm these calculations are correct when things are deriving from the else case
        let se_wall = (pawn / 9) * 8 + (pawn % 9);
        let sw_wall = if se_wall > 0 { se_wall - 1 } else { 0 };
        let nw_wall = if se_wall > 9 { se_wall - 9 } else { 0 };
        let ne_wall = nw_wall + 1;
        // TODO hopping over pawns
        // north
        if pawn > 8
            && (pawn % 9 == 0 || ((1 << nw_wall) & self.hwalls) == 0)
            && ((pawn + 1) % 9 == 0 || ((1 << ne_wall) & self.hwalls) == 0)
        {
            let mut child = self.clone();
            child.pawns[turn] = pawn - 9;
            child.turn = child.turn.other();
            moves.push(child);
        }
        // south
        if pawn < 72
            && (pawn % 9 == 0 || ((1 << sw_wall) & self.hwalls) == 0)
            && ((pawn + 1) % 9 == 0 || ((1 << se_wall) & self.hwalls) == 0)
        {
            let mut child = self.clone();
            child.pawns[turn] = pawn + 9;
            child.turn = child.turn.other();
            moves.push(child);
        }
        // east
        if (pawn + 1) % 9 != 0
            && (pawn < 9 || ((1 << ne_wall) & self.vwalls) == 0)
            && (pawn > 71 || ((1 << se_wall) & self.vwalls) == 0)
        {
            let mut child = self.clone();
            child.pawns[turn] = pawn + 1;
            child.turn = child.turn.other();
            moves.push(child);
        }
        // west
        if (pawn) % 9 != 0
            && (pawn < 9 || ((1 << nw_wall) & self.vwalls) == 0)
            && (pawn > 71 || ((1 << sw_wall) & self.vwalls) == 0)
        {
            let mut child = self.clone();
            child.pawns[turn] = pawn - 1;
            child.turn = child.turn.other();
            moves.push(child);
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
            if i == 0 || ((wall_bit >> 1) & self.hwalls) == 0 {
                if i == 63 || ((wall_bit << 1) & self.hwalls == 0) {
                    let mut child = self.clone();
                    child.hwalls |= wall_bit;
                    child.remaining_walls[turn] -= 1;
                    child.turn = child.turn.other();
                    moves.push(child);
                }
            }
            if i == 0 || ((wall_bit >> 1) & self.vwalls) == 0 {
                if i == 63 || ((wall_bit << 1) & self.vwalls == 0) {
                    let mut child = self.clone();
                    child.vwalls |= wall_bit;
                    child.remaining_walls[turn] -= 1;
                    child.turn = child.turn.other();
                    moves.push(child);
                }
            }
        }

        moves
    }

    pub fn print(self) {
        for row in 0..9 {
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
                    let sqnum = row * 9 + col;
                    let se_wall = (sqnum / 9) * 8 + (sqnum % 9);
                    let sw_wall = if se_wall > 1 { se_wall - 1 } else { 0 };
                    if ((sqnum + 1) % 9 != 0 && (self.hwalls & (1 << se_wall)) > 0)
                        || (sqnum % 9 != 0) && (self.hwalls & (1 << sw_wall) > 0)
                    {
                        eprint!("#   ");
                    } else {
                        eprint!("    ");
                    }
                }
            }
            eprintln!();
        }
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
}
