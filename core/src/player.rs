use crate::board::{Board, Move, Position, BOARD_HEIGHT, GhostID};
use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerID {
    P1,
    P2,
}

impl PlayerID {
    fn init_yrange(&self) -> Range<usize> {
        match self {
            PlayerID::P1 => 0..2,
            PlayerID::P2 => BOARD_HEIGHT - 2..BOARD_HEIGHT,
        }
    }
    pub fn init(&self, mut f: impl FnMut(Position, GhostID))
    {
        let mut cnt = 0;
        for x in 1..=4 {
            for y in self.init_yrange() {
                let pos = Position::new(x as i8, y as i8);
                let ghost = match self {
                    PlayerID::P1 => GhostID::from_u8(7 - cnt),
                    PlayerID::P2 => GhostID::from_u8(cnt),
                }.unwrap();
                f(pos, ghost);
                cnt += 1;
            }
        }
    }
    pub fn rev(&self) -> PlayerID {
        match self {
            PlayerID::P1 => PlayerID::P2,
            PlayerID::P2 => PlayerID::P1,
        }
    }
}

pub trait Player {
    type Error;
    fn id(&self) -> PlayerID;
    fn init(&mut self, id: PlayerID) -> Result<[Position; 4], Self::Error>;
    fn board(&self) -> &Board;
    fn step(&mut self, board: Board) -> Result<Move, Self::Error>;
    fn close(&mut self, _victory: bool) {}
}




