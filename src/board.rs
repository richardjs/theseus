#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Player {
    White = 0,
    Black = 1,
}
use Player::*;

impl Player {
    pub fn other(&self) -> Player {
        match self {
            White => Black,
            Black => White,
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        White
    }
}

#[derive(Debug)]
pub enum Direction {
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

    fn move_sqnum(&self, sqnum: u8) -> u8 {
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

    shortest_path_cache: [Option<Vec<u8>>; 2],
}

impl Board {
    pub fn new() -> Board {
        Board {
            pawns: [sqnum_for_coord('e', 9), sqnum_for_coord('e', 1)],
            remaining_walls: [10, 10],
            hwalls: 0,
            vwalls: 0,
            turn: White,
            shortest_path_cache: [None, None],
        }
    }

    pub fn from_tqbn(tqbn: &str) -> Board {
        assert!(tqbn.len() == 73);
        let tqbn: Vec<_> = tqbn.chars().collect();

        let mut hwalls = 0;
        let mut vwalls = 0;
        for (i, c) in tqbn[0..64].into_iter().enumerate() {
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
                sqnum_for_coord(tqbn[65], tqbn[66].to_digit(10).unwrap() as u8),
                sqnum_for_coord(tqbn[69], tqbn[70].to_digit(10).unwrap() as u8),
            ],
            remaining_walls: [
                tqbn[67..69]
                    .iter()
                    .collect::<String>()
                    .parse::<u8>()
                    .unwrap(),
                tqbn[71..73]
                    .iter()
                    .collect::<String>()
                    .parse::<u8>()
                    .unwrap(),
            ],
            hwalls,
            vwalls,
            turn: match tqbn[64] {
                '1' => White,
                '2' => Black,
                _ => panic!(),
            },
            shortest_path_cache: [None, None],
        }
    }

    pub fn turn(&self) -> Player {
        self.turn
    }

    pub fn pawns(&self) -> [u8; 2] {
        self.pawns
    }

    pub fn turn_pawn(&self) -> u8 {
        self.pawns[self.turn as usize]
    }

    pub fn other_pawn(&self) -> u8 {
        self.pawns[self.turn.other() as usize]
    }

    pub fn remaining_walls(&self) -> [u8; 2] {
        self.remaining_walls
    }

    pub fn winner(&self) -> Option<Player> {
        if self.pawns[White as usize] < 9 {
            return Some(White);
        }
        if self.pawns[Black as usize] > 71 {
            return Some(Black);
        }
        None
    }

    pub fn can_win(&self) -> bool {
        let possible_win_row = match self.turn {
            White => {
                self.turn_pawn() < 18
                    || (self.turn_pawn() < 26 && self.other_pawn() + 9 == self.turn_pawn())
            }
            Black => {
                self.turn_pawn() > 62
                    || (self.turn_pawn() > 53 && self.other_pawn() == self.turn_pawn() + 9)
            }
        };
        if !possible_win_row {
            return false;
        }

        for child in self.moves_detailed(true, false, true) {
            if child.winner().is_some() {
                return true;
            }
        }
        return false;
    }

    pub fn is_open(&self, sqnum: u8, direction: &Direction) -> bool {
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

    pub fn moves_detailed(
        &self,
        moves_only: bool,
        validate_paths: bool,
        return_wins: bool,
    ) -> Vec<Board> {
        let turn = self.turn as usize;
        let other = self.turn.other() as usize;
        let pawn = self.pawns[turn];

        let mut moves = vec![];

        // pawn movements
        for direction in [North, South, East, West].iter() {
            if !self.is_open(pawn, direction) {
                continue;
            }

            let mut child = self.clone();
            child.pawns[turn] = direction.move_sqnum(pawn);
            child.turn = child.turn.other();

            let child_pawn = child.pawns[turn];
            if let Some(cache) = &mut child.shortest_path_cache[turn] {
                if *cache.first().unwrap() == child_pawn {
                    cache.remove(0);
                    child.shortest_path_cache[turn] = Some(cache.to_vec());
                } else {
                    child.shortest_path_cache[turn] = None;
                }
            }

            // jumping
            if child.pawns[turn] == child.pawns[other] {
                if self.is_open(child.pawns[turn], direction) {
                    child.pawns[turn] = direction.move_sqnum(child.pawns[turn]);
                    if return_wins && child.winner().is_some() {
                        return vec![child];
                    }
                    moves.push(child);
                } else {
                    if self.is_open(child.pawns[turn], &direction.left()) {
                        // clone the child in case we also can jump to the right
                        let mut child = child.clone();
                        child.pawns[turn] = direction.left().move_sqnum(child.pawns[turn]);
                        if return_wins && child.winner().is_some() {
                            return vec![child];
                        }
                        moves.push(child);
                    }
                    if self.is_open(child.pawns[turn], &direction.right()) {
                        child.pawns[turn] = direction.right().move_sqnum(child.pawns[turn]);
                        if return_wins && child.winner().is_some() {
                            return vec![child];
                        }
                        moves.push(child);
                    }
                }
            } else {
                if return_wins && child.winner().is_some() {
                    return vec![child];
                }
                moves.push(child);
            }
        }

        // wall placements
        // we're going to start with a fairly naive algorithm, and optimize later
        if moves_only || self.remaining_walls[turn] == 0 {
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
                child.shortest_path_cache[White as usize] = None;
                child.shortest_path_cache[Black as usize] = None;
                if !validate_paths || child.paths_exist() {
                    moves.push(child);
                }
            }
            if (i < 8 || ((wall_bit >> 8) & self.vwalls) == 0)
                && (i > 55 || ((wall_bit << 8) & self.vwalls == 0))
            {
                let mut child = self.clone();
                child.vwalls |= wall_bit;
                child.remaining_walls[turn] -= 1;
                child.turn = child.turn.other();
                child.shortest_path_cache[White as usize] = None;
                child.shortest_path_cache[Black as usize] = None;
                if !validate_paths || child.paths_exist() {
                    moves.push(child);
                }
            }
        }

        moves
    }

    pub fn moves(&self) -> Vec<Board> {
        self.moves_detailed(false, true, false)
    }

    pub fn paths_exist(&self) -> bool {
        let mut white_path = false;
        let mut queue = vec![self.pawns[White as usize]];
        let mut crumbs = [false; 81];
        crumbs[queue[0] as usize] = true;
        while queue.len() > 0 && !white_path {
            let sqnum = queue.pop().unwrap();

            for direction in [North, South, East, West].iter() {
                if !self.is_open(sqnum, direction) {
                    continue;
                }
                let move_sqnum = direction.move_sqnum(sqnum);
                if move_sqnum < 9 {
                    white_path = true;
                    break;
                }
                if crumbs[move_sqnum as usize] {
                    continue;
                }
                queue.insert(0, move_sqnum);
                crumbs[move_sqnum as usize] = true;
            }
        }

        let mut black_path = false;
        let mut queue = vec![self.pawns[Black as usize]];
        let mut crumbs = [false; 81];
        crumbs[queue[0] as usize] = true;
        while queue.len() > 0 && !black_path {
            let sqnum = queue.pop().unwrap();

            for direction in [North, South, East, West].iter() {
                if !self.is_open(sqnum, direction) {
                    continue;
                }
                let move_sqnum = direction.move_sqnum(sqnum);
                if move_sqnum > 71 {
                    black_path = true;
                    break;
                }
                if crumbs[move_sqnum as usize] {
                    continue;
                }
                queue.insert(0, move_sqnum);
                crumbs[move_sqnum as usize] = true;
            }
        }

        white_path && black_path
    }

    pub fn shortest_path(&mut self, player: Player) -> Vec<u8> {
        // TODO test shortest path cache, and optimize it (for example, wall placement doesn't invalidate it if it doesn't intersect with it)
        if let Some(cache) = self.shortest_path_cache[player as usize].clone() {
            return cache;
        }

        let mut queue = vec![vec![self.pawns[player as usize]]];
        let mut crumbs = [false; 81];
        crumbs[queue[0][0] as usize] = true;
        while queue.len() > 0 {
            let path = queue.pop().unwrap();
            let sqnum = path.last().unwrap();

            for direction in [North, South, East, West].iter() {
                if !self.is_open(*sqnum, direction) {
                    continue;
                }
                let move_sqnum = direction.move_sqnum(*sqnum);
                if player == White {
                    if move_sqnum < 9 {
                        let mut path = path.clone();
                        path.push(move_sqnum);
                        path.remove(0);
                        self.shortest_path_cache[player as usize] = Some(path.clone());
                        return path;
                    }
                } else {
                    if move_sqnum > 71 {
                        let mut path = path.clone();
                        path.push(move_sqnum);
                        path.remove(0);
                        self.shortest_path_cache[player as usize] = Some(path.clone());
                        return path;
                    }
                }
                if crumbs[move_sqnum as usize] {
                    continue;
                }
                let mut new_path = path.clone();
                new_path.push(move_sqnum);
                queue.insert(0, new_path);
                crumbs[move_sqnum as usize] = true;
            }
        }
        Vec::new()
    }

    /// returns an array of steps needed to reach each sqnum
    pub fn walk_paths(&self, player: Player) -> [u32; 81] {
        let pawn = self.pawns()[player as usize];
        let mut walk = vec![self.pawns()[player as usize]];
        let mut counts = [0; 81];
        let mut steps = 1;

        while walk.len() > 0 {
            let mut next_walk = Vec::new();
            for sqnum in &walk {
                for direction in [North, South, East, West].iter() {
                    if !self.is_open(*sqnum, direction) {
                        continue;
                    }

                    let move_sqnum = direction.move_sqnum(*sqnum);

                    if counts[move_sqnum as usize] > 0 || move_sqnum == pawn {
                        continue;
                    }

                    counts[move_sqnum as usize] = steps;

                    if (player == White && move_sqnum < 9) || (player == Black && move_sqnum > 71) {
                        continue;
                    }

                    next_walk.push(move_sqnum);
                }
            }
            walk = next_walk;
            steps += 1;
        }

        counts
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
	s.push_str(&format!("{} to move\n", match self.turn() {
	    White => "white",
	    Black => "black",
	}));
        s.push_str("  a   b   c   d   e   f   g   h   i\n");
        for row in 0..9 {
            s.push_str(&format!("{} ", row + 1));
            for col in 0..9 {
                let sqnum = row * 9 + col;
                let se_wall = (sqnum / 9) * 8 + (sqnum % 9);
                let ne_wall = if se_wall > 7 { se_wall - 8 } else { 0 };
                if self.pawns[White as usize] == sqnum {
                    s.push('W');
                } else if self.pawns[Black as usize] == sqnum {
                    s.push('B');
                } else {
                    s.push('.');
                }
                if col != 8 {
                    if (sqnum > 8 && (self.vwalls & (1 << ne_wall)) > 0)
                        || (sqnum < 72 && (self.vwalls & (1 << se_wall)) > 0)
                    {
                        s.push_str(" # ");
                    } else {
                        s.push_str("   ");
                    }
                }
            }
            s.push('\n');
            if row < 8 {
                for col in 0..9 {
                    s.push_str("  ");
                    let sqnum = row * 9 + col;
                    let se_wall = (sqnum / 9) * 8 + (sqnum % 9);
                    let sw_wall = if se_wall > 1 { se_wall - 1 } else { 0 };
                    if ((sqnum + 1) % 9 != 0 && (self.hwalls & (1 << se_wall)) > 0)
                        || (sqnum % 9 != 0) && (self.hwalls & (1 << sw_wall) > 0)
                    {
                        s.push_str("# ");
                    } else {
                        s.push_str("  ");
                    }
                }
                s.push('\n');
            }
        }
        s
    }

    pub fn print(&self) {
        eprint!("{}", self.to_string());
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
        board.pawns[White as usize] = 12;
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
        board.pawns[White as usize] = 0;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.pawns[White as usize] = 8;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.pawns[White as usize] = 72;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
        board.pawns[White as usize] = 80;
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 2);
    }

    #[test]
    fn load_tqbn() {
        let board = Board::from_tqbn(&String::from(
            "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn1e910e110",
        ));
        assert_eq!(board.pawns[0], 76);
        assert_eq!(board.pawns[1], 4);
        assert_eq!(board.remaining_walls[0], 10);
        assert_eq!(board.remaining_walls[1], 10);
        assert_eq!(board.hwalls, 0);
        assert_eq!(board.vwalls, 0);
        assert_eq!(board.turn, White);
    }

    #[test]
    fn vertical_wall_place_bug() {
        let board = Board::from_tqbn(
            "nnnnnnnnnnvnnnnnnnhnnnnnnnnnnhnnnnvnvnnvnnnnhnnnhnnnnnnnnnnnhnnn2e905e106",
        );
        for child in board.moves() {
            assert_ne!(board.move_string_to(&child), "c1v");
        }
        let board = Board::from_tqbn(
            "nnnhvnnnnnnnnnnnnnnnnnnnnnvnvnnnnnnnnnvnnnvnnnnnnnnnvnnnnnnvnnnn1e906e106",
        );
        for child in board.moves() {
            assert_ne!(board.move_string_to(&child), "e3v");
        }
    }

    #[test]
    fn simple_pawn_jumping() {
        // jump to the north
        let board = Board::from_tqbn(&String::from(
            "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2e410e510",
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
            "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2e510e410",
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
            "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2e410d410",
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
            "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2d410e410",
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

    #[test]
    fn blocked_pawn_jumping() {
        let board = Board::from_tqbn(&String::from(
            "nnnnnnnnnnnnnhnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2f309f410",
        ));
        for child in only_pawn_moves(&board, board.moves()) {
            assert_ne!(board.move_string_to(&child), "f2");
        }
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 5);

        let board = Board::from_tqbn(&String::from(
            "nnnnnnnnnnnnvhnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2f309f410",
        ));
        for child in only_pawn_moves(&board, board.moves()) {
            assert_ne!(board.move_string_to(&child), "e3");
        }
        assert_eq!(only_pawn_moves(&board, board.moves()).len(), 4);
    }

    #[test]
    fn keep_paths_open() {
        let board = Board::from_tqbn(&String::from(
            "nnnnnnnnhnhnhnhnnnnnnnnhnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn2e909e110",
        ));
        for child in board.moves() {
            assert_ne!(board.move_string_to(&child), "h2v");
        }
    }

    #[test]
    fn basic_shortest_path_cache_usage() {
        let mut board = Board::new();
        assert_eq!(board.shortest_path_cache[0], None);
        assert_eq!(board.shortest_path_cache[1], None);
        board.shortest_path(White);
        assert_ne!(board.shortest_path_cache[0], None);
        assert_eq!(board.shortest_path_cache[1], None);
        board.shortest_path(Black);
        assert_ne!(board.shortest_path_cache[0], None);
        assert_ne!(board.shortest_path_cache[1], None);
        board = board.moves()[0].clone();
        assert_ne!(board.shortest_path_cache[0], None);
        assert_ne!(board.shortest_path_cache[1], None);
        board = board.moves()[0].clone();
        assert_ne!(board.shortest_path_cache[0], None);
        assert_ne!(board.shortest_path_cache[1], None);
        board = board.moves()[2].clone();
        assert_eq!(board.shortest_path_cache[0], None);
        assert_ne!(board.shortest_path_cache[1], None);
        board = board.moves()[50].clone();
        assert_eq!(board.shortest_path_cache[0], None);
        assert_eq!(board.shortest_path_cache[1], None);
    }
}
