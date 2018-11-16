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
    (n as usize) < max
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Hash, Add, Sub)]
#[display(fmt = "({}, {})", x, y)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    const INVALID: usize = BOARD_HEIGHT * BOARD_WIDTH;
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
        if x < 0 || y < 0 {
            return Self::INVALID;                
        }
        x as usize + y as usize * BOARD_WIDTH
    }
    pub fn is_escape(&self, player: PlayerID) -> bool {
        let &Position { x, y } = self;
        if x != 0 && x != BOARD_WIDTH as i8 - 1 {
            return false;
        }
        match player {
            PlayerID::P1 => y == BOARD_HEIGHT as i8 - 1,
            PlayerID::P2 => y == 0,
        }
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
    pub fn iter() -> impl Iterator<Item = Self> {
        const ITER: [Direction; 4] = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        ITER.iter().map(|&x| x)
    }
    pub fn is_ordinal(&self) -> bool {
        match self {
            Direction::Up | Direction::Down => false,
            _ => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move {
    pub pos: Position,
    pub direction: Direction,
}

impl Move {
    pub fn to(self) -> Position {
        let Move { pos, direction } = self;
        pos + direction.to_pos()
    }
    pub fn to_indices(self) -> (usize, usize) {
        (self.pos.to_index(), self.to().to_index())
    }
    pub fn can_escape(&self, player: PlayerID) -> bool {
        self.pos.is_escape(player) && self.direction.is_ordinal()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GhostID {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl GhostID {
    pub fn from_u8(u: u8) -> Option<Self> {
        Some(match u {
            0 => GhostID::A,
            1 => GhostID::B,
            2 => GhostID::C,
            3 => GhostID::D,
            4 => GhostID::E,
            5 => GhostID::F,
            6 => GhostID::G,
            7 => GhostID::H,
            _ => return None,
        })
    }
    pub fn as_u8(&self) -> u8 {
        match self {
            GhostID::A => 0,
            GhostID::B => 1,
            GhostID::C => 2,
            GhostID::D => 3,
            GhostID::E => 4,
            GhostID::F => 5,
            GhostID::G => 6,
            GhostID::H => 7,
        }
    }
}

impl fmt::Display for GhostID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ghost {
    Unknown,
    Red,
    Blue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Owned(OwnedCell),
    Empty,
}

impl Cell {
    pub fn is_owned(&self) -> bool {
        match self {
            Cell::Owned(_) => true,
            Cell::Empty => false,
        }
    }
    pub fn is_empty(&self) -> bool {
        !self.is_owned()
    }
    pub fn owned(kind: Ghost, owner: PlayerID, id: GhostID) -> Self {
        let mut cell = OwnedCell(0);
        cell.set_ghost(kind);
        cell.set_owner(owner);
        cell.set_id(id);
        Cell::Owned(cell)
    }
    pub fn owner(&self) -> Option<PlayerID> {
        match self {
            Cell::Owned(o) => Some(o.owner()),
            Cell::Empty => None,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    fn set_ghost(&mut self, ghost: Ghost) {
        let offset = match ghost {
            Ghost::Unknown => 0,
            Ghost::Red => 1,
            Ghost::Blue => 2,
        };
        self.0 |= offset;
    }
    fn set_owner(&mut self, owner: PlayerID) {
        let offset = match owner {
            PlayerID::P1 => 0,
            PlayerID::P2 => 1,
        };
        self.0 |= offset << Self::OWNER_OFFSET;
    }
    fn set_id(&mut self, id: GhostID) {
        let offset = id.as_u8();
        self.0 |= offset << Self::ID_OFFSET;
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
        GhostID::from_u8(self.get_mask(Self::ID_MASK, Self::ID_OFFSET)).unwrap()
    }
}

impl fmt::Display for OwnedCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (kind, owner) = (self.ghost(), self.owner());
        match owner {
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
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Owned(cell) => write!(f, "{}", cell),
            Cell::Empty => write!(f, "  "),
        }
    }
}

pub trait AsCell: Copy + Default + Sized {
    fn as_cell(&self) -> &Cell; 
    fn as_cell_mut(&mut self) -> &mut Cell;
    fn to_cell(&self) -> Cell;
}

impl AsCell for Cell {
    fn as_cell(&self) -> &Cell {
        self
    }
    fn as_cell_mut(&mut self) -> &mut Cell {
        self
    }
    fn to_cell(&self) -> Cell {
        *self
    }
}

#[derive(Clone)]
pub struct GenericBoard<C: Sized> {
    inner: [C; BOARD_HEIGHT * BOARD_WIDTH],    
}

pub type Board = GenericBoard<Cell>;
    
impl<C: AsCell> Default for GenericBoard<C> {
    fn default() -> Self {
        GenericBoard {
            inner: [C::default(); BOARD_HEIGHT * BOARD_WIDTH],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MoveResult {
    Ok,
    Win,
    Err,
}

impl From<bool> for MoveResult {
    fn from(b: bool) -> Self {
        if b {
            MoveResult::Ok
        } else {
            MoveResult::Err
        }
    }
}
    
pub struct Diff {
    pub pos: Position,
    pub before: Cell,
    pub after: Cell,
}

impl Diff {
    pub fn into_transition(self) -> Transition {
        let Diff {
            pos,
            before,
            after,
        } = self;
        match before {
            Cell::Owned(before) => match after {
                Cell::Owned(_) => Transition::Lost(before),
                Cell::Empty => {
                    let owner = before.owner();
                    if before.ghost() == Ghost::Blue && pos.is_escape(owner) {
                        Transition::End(owner)
                    } else {
                        Transition::None
                    }
                }
            }
            Cell::Empty => Transition::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Transition {
    Lost(OwnedCell),
    End(PlayerID),
    None,
}
    
impl<C: AsCell> GenericBoard<C> {
    pub fn iter() -> RectRange<i8> {
        RectRange::zero_start(BOARD_WIDTH as i8, BOARD_HEIGHT as i8).unwrap()
    }
    pub fn init_for_player(red_pos: [Position; 4], player: PlayerID) -> Option<Self> {
        let mut board = Self::default();
        let mut cnt = 0;
        player.init(|pos, id| {
            let ghost = if red_pos.contains(&pos) {
                cnt += 1;
                Ghost::Red
            } else {
                Ghost::Blue
            };
            *board[pos].as_cell_mut() = Cell::owned(ghost, player, id);
        });
        if cnt != 4 {
            return None;
        }
        player.rev().init(|pos, id| {
            *board[pos].as_cell_mut() = Cell::owned(Ghost::Unknown, player.rev(), id);
        });
        Some(board)
    }
    pub fn transit(&mut self, mov: Move) -> Result<Transition, ErrorKind> {
        if !mov.pos.is_valid() {
            return ErrorKind::InvalidMove(mov).into();
        }
        let (from, to) = mov.to_indices();
        let from = match self.inner[from].to_cell() {
            Cell::Owned(o) => o,
            Cell::Empty => return ErrorKind::InvalidMove(mov).into(),
        };
        match self.inner.get_mut(to) {
            Some(cell) => {
                let before = cell.to_cell();
                *cell.as_cell_mut() = Cell::Owned(from);
                match before {
                    Cell::Owned(o) => Ok(Transition::Lost(o)),
                    Cell::Empty => Ok(Transition::None),
                }
            }
            None => {
                if mov.can_escape(from.owner()) && from.ghost() == Ghost::Blue {
                    Ok(Transition::End(from.owner()))
                } else {
                    ErrorKind::InvalidMove(mov).into()
                }
            }
        }
    }
    pub fn can_move(&self, mov: Move) -> MoveResult {
        if !mov.pos.is_valid() {
            return MoveResult::Err;
        }
        let to_p = mov.to();
        let (from, to) = mov.to_indices();
        let (owner, ghost) = match self.inner[from].to_cell() {
            Cell::Owned(o) => (o.owner(), o.ghost()),
            Cell::Empty => return MoveResult::Err,
        };
        if to_p.is_valid() {
            match self.inner[to].to_cell() {
                Cell::Empty => true,
                Cell::Owned(o) => o.owner() != owner,
            }.into()
        } else {
            if mov.can_escape(owner) && ghost == Ghost::Blue {
                MoveResult::Win
            } else {
                MoveResult::Err                    
            }                
        }
    }
    pub fn diff(&self, other: &Self) -> Vec<Diff> {
        let mut out = vec![];
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                let idx = x + y * BOARD_WIDTH;
                let (self_, other) = (self.inner[idx].to_cell(), other.inner[idx].to_cell());
                if self_ != other {
                    out.push(Diff {
                        pos: Position::new(x as i8, y as i8),
                        before: self_,
                        after: other,
                    });
                }
            }
        }
        out
    }
}

impl<C: AsCell> Index<Position> for GenericBoard<C> {
    type Output = C;
    fn index(&self, pos: Position) -> &C {
        debug_assert!(pos.is_valid());
        &self.inner[pos.to_index()]
    }
}

impl<C: AsCell> IndexMut<Position> for GenericBoard<C> {
    fn index_mut(&mut self, pos: Position) -> &mut C {
        debug_assert!(pos.is_valid());
        &mut self.inner[pos.to_index()]
    }
}

impl<C: AsCell> fmt::Debug for GenericBoard<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "player1")?;
        writeln!(f, "  0 1 2 3 4 5")?;
        for y in 0..BOARD_HEIGHT {
            write!(f, "{}", y)?;
            for x in 0..BOARD_WIDTH {
                write!(f, "{} ", self[Position::new(x as i8, y as i8)].as_cell())?;
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
    let board = Board::init_for_player(pos, PlayerID::P1).unwrap();
    println!("{:?}", board);
}
