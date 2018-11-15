use crate::error::ErrorKind;
use std::fmt;
use std::ops::{Index, IndexMut};
use yansi::Paint;
pub const BOARD_HEIGHT: usize = 6;
pub const BOARD_WIDTH: usize = 6;
use crate::player::PlayerID;
use rect_iter::RectRange;

fn check(n: i8, max: usize) -> bool {
    if n < 0 {
        return false;
    }
    let n = n as usize;
    n < max
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Hash, Add, Sub)]
#[display(fmt = "({}, {})", x, y)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    pub fn in_init_area(&self) -> Option<PlayerID> {
        let Position { x, y } = *self;
        if !(1 <= x && x < BOARD_WIDTH as i8 - 1) {
            return None;
        }
        if 0 <= y && y < 2 {
            Some(PlayerID::P1)
        } else if BOARD_HEIGHT as i8 - 2 <= y && y < BOARD_HEIGHT as i8 {
            Some(PlayerID::P2)
        } else {
            None
        }
    }
    pub fn new(x: i8, y: i8) -> Self {
        Position { x, y }
    }
    pub fn is_valid(self) -> bool {
        check(self.x, BOARD_WIDTH) && check(self.y, BOARD_HEIGHT)
    }
    pub fn to_index(self) -> usize {
        let Position { x, y } = self;
        x as usize + y as usize * BOARD_WIDTH
    }
}

impl From<(i8, i8)> for Position {
    fn from((x, y): (i8, i8)) -> Position {
        Position::new(x, y)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_pos(self) -> Position {
        match self {
            Direction::Up => Position::new(0, -1),
            Direction::Down => Position::new(0, 1),
            Direction::Left => Position::new(-1, 0),
            Direction::Right => Position::new(1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move {
    pub from: Position,
    pub direction: Direction,
}

impl Move {
    pub fn to(self) -> Position {
        let Move { from, direction } = self;
        from + direction.to_pos()
    }
    pub fn to_indices(self) -> (usize, usize) {
        (self.from.to_index(), self.to().to_index())
    }
}

pub enum GhostId {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ghost {
    Unknown,
    Red,
    Blue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Owned { kind: Ghost, owner: PlayerID },
    Empty,
}

impl Cell {
    pub fn owned(kind: Ghost, owner: PlayerID) -> Self {
        Cell::Owned { kind, owner }
    }
}

pub struct OwnedCell(u8);

impl OwnedCell {
    const GHOST_MASK: u8 = 0b00000011;
    const OWNER_MASK: u8 = 0b00000100;
    const ID_MASK: u8 = 0b00111000;
    const GHOST_OFFSET: usize = 0;
    const OWNER_OFFSET: usize = 2;
    const ID_OFFSET: usize = Self::OWNER_OFFSET + 1;
    #[inline(always)]
    fn get_mask(&self, mask: u8, offset: usize) -> u8 {
        (self.0 & mask) >> offset
    }
    pub fn ghost(&self) -> Ghost {
        match self.get_mask(Self::GHOST_MASK, Self::GHOST_OFFSET) {
            0 => Ghost::Unknown,
            1 => Ghost::Red,
            2 => Ghost::Blue,
            _ => unreachable!(),
        }
    }
    pub fn owner(&self) -> PlayerID {
        match self.get_mask(Self::OWNER_MASK, Self::OWNER_OFFSET) {
            0 => PlayerID::P1,
            1 => PlayerID::P2,
            _ => unreachable!(),
        }
    }
    pub fn id(&self) -> GhostID {
        match self.get_mask(Self::GHOST_MASK, Self::GHOST_OFFSET) {
            0 => PlayerID::P1,
            1 => PlayerID::P2,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for OwnedCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Owned { kind, owner } => match owner {
                PlayerID::P1 => match kind {
                    Ghost::Red => write!(f, "{}", Paint::red("▽")),
                    Ghost::Blue => write!(f, "{}", Paint::blue("▽")),
                    Ghost::Unknown => write!(f, "{}", Paint::white("▽")),
                },
                PlayerID::P2 => match kind {
                    Ghost::Red => write!(f, "{}", Paint::red("△")),
                    Ghost::Blue => write!(f, "{}", Paint::blue("△")),
                    Ghost::Unknown => write!(f, "{}", Paint::white("△")),
                },
            },
            Cell::Empty => write!(f, "  "),
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Owned { kind, owner } => match owner {
                PlayerID::P1 => match kind {
                    Ghost::Red => write!(f, "{}", Paint::red("▽")),
                    Ghost::Blue => write!(f, "{}", Paint::blue("▽")),
                    Ghost::Unknown => write!(f, "{}", Paint::white("▽")),
                },
                PlayerID::P2 => match kind {
                    Ghost::Red => write!(f, "{}", Paint::red("△")),
                    Ghost::Blue => write!(f, "{}", Paint::blue("△")),
                    Ghost::Unknown => write!(f, "{}", Paint::white("△")),
                },
            },
            Cell::Empty => write!(f, "  "),
        }
    }
}

#[derive(Clone)]
pub struct Board {
    /// 72 byte
    inner: [Cell; BOARD_HEIGHT * BOARD_WIDTH],
}

impl Default for Board {
    fn default() -> Self {
        Board {
            inner: [Cell::Empty; BOARD_HEIGHT * BOARD_WIDTH],
        }
    }
}

impl Board {
    pub fn iter() -> RectRange<i8> {
        RectRange::zero_start(BOARD_WIDTH as i8, BOARD_HEIGHT as i8).unwrap()
    }
    fn init_with(&mut self, red_pos: [Position; 4], player: PlayerID) {
        for &pos in red_pos.iter() {
            self[pos] = Cell::owned(Ghost::Red, player);
        }
        for x in 1..=4 {
            for y in player.init_range() {
                let pos = Position::new(x, y as i8);
                let ghost = if red_pos.contains(&pos) {
                    Ghost::Red
                } else {
                    Ghost::Blue
                };
                self[pos] = Cell::owned(ghost, player);
            }
        }
        for x in 1..=4 {
            for y in player.init_range_rev() {
                let pos = Position::new(x, y as i8);
                self[pos] = Cell::owned(Ghost::Unknown, player.rev());
            }
        }
    }
    pub fn init_for_player(red_pos: [Position; 4]) -> Option<Self> {
        let player = {
            let mut iter = red_pos.iter();
            let player = iter.next().and_then(|p| p.in_init_area())?;
            for p in iter {
                if p.in_init_area()? != player {
                    return None;
                }
            }
            player
        };
        let mut board = Board::default();
        board.init_with(red_pos, player);
        Some(board)
    }
    pub fn can_move(&self, mov: Move) -> Result<bool, ErrorKind> {
        if !mov.from.is_valid() {
            return Err(ErrorKind::IndexError(mov.from));
        }
        let to = mov.to();
        if !to.is_valid() {
            return Err(ErrorKind::IndexError(to));
        }
        Ok(self.can_move_nocheck(mov))
    }
    pub fn can_move_nocheck(&self, mov: Move) -> bool {
        let (from, to) = mov.to_indices();
        let owner1 = match self.inner[from] {
            Cell::Owned { kind: _, owner } => owner,
            Cell::Empty => return false,
        };
        match self.inner[to] {
            Cell::Empty => return true,
            Cell::Owned { owner: owner2, .. } => owner1 != owner2,
        }
    }
}

impl Index<Position> for Board {
    type Output = Cell;
    fn index(&self, pos: Position) -> &Cell {
        debug_assert!(pos.is_valid());
        &self.inner[pos.to_index()]
    }
}

impl IndexMut<Position> for Board {
    fn index_mut(&mut self, pos: Position) -> &mut Cell {
        debug_assert!(pos.is_valid());
        &mut self.inner[pos.to_index()]
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "player1")?;
        writeln!(f, "  0 1 2 3 4 5")?;
        for y in 0..BOARD_HEIGHT {
            write!(f, "{}", y)?;
            for x in 0..BOARD_WIDTH {
                write!(f, "{} ", self[Position::new(x as i8, y as i8)])?;
            }
            writeln!(f)?;
        }
        writeln!(f, "player2")
    }
}

#[test]
fn print_board() {
    let pos = [
        Position::new(1, 0),
        Position::new(2, 1),
        Position::new(3, 0),
        Position::new(4, 1),
    ];
    let board = Board::init_for_player(pos);
    println!("{:?}", board);
}
