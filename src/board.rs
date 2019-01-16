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

        // Pawn movements
        let se_wall = (pawn / 9) * 8 + (pawn % 9);
        let sw_wall = if se_wall > 0 { se_wall - 1 } else { 0 };
        let nw_wall = if se_wall > 9 { se_wall - 10 } else { 0 };
        let ne_wall = nw_wall + 1;
        // TODO wall blocking
        // TODO hopping over pawns
        // North
        if pawn > 8
            && (pawn % 9 == 0 || ((1 << nw_wall) & self.hwalls) == 0)
            && ((pawn + 1) % 9 == 0 || ((1 << ne_wall) & self.hwalls) == 0)
        {
            let mut child = self.clone();
            child.pawns[turn] = pawn - 9;
            child.turn = child.turn.other();
            moves.push(child);
        }
        // South
        if pawn < 72
            && (pawn % 9 == 0 || ((1 << sw_wall) & self.hwalls) == 0)
            && ((pawn + 1) % 9 == 0 || ((1 << se_wall) & self.hwalls) == 0)
        {
            let mut child = self.clone();
            child.pawns[turn] = pawn + 9;
            child.turn = child.turn.other();
            moves.push(child);
        }
        // East
        if (pawn + 1) % 9 != 0
            && (pawn < 9 || ((1 << ne_wall) & self.vwalls) == 0)
            && (pawn > 71 || ((1 << se_wall) & self.vwalls) == 0)
        {
            let mut child = self.clone();
            child.pawns[turn] = pawn + 1;
            child.turn = child.turn.other();
            moves.push(child);
        }
        // West
        if (pawn + 1) % 9 != 0
            && (pawn < 9 || ((1 << nw_wall) & self.vwalls) == 0)
            && (pawn > 71 || ((1 << sw_wall) & self.vwalls) == 0)
        {
            let mut child = self.clone();
            child.pawns[turn] = pawn - 1;
            child.turn = child.turn.other();
            moves.push(child);
        }

        // Wall placements
        // We're going to start with a fairly naive algorithm, and optimize later
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
}
