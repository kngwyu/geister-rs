use crate::board::{Board, Move, Position, BOARD_HEIGHT};
use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerID {
    P1,
    P2,
}

impl PlayerID {
    pub fn init_range(&self) -> Range<usize> {
        match self {
            PlayerID::P1 => 0..2,
            PlayerID::P2 => BOARD_HEIGHT - 2..BOARD_HEIGHT,
        }
    }
    pub fn init_range_rev(&self) -> Range<usize> {
        match self {
            PlayerID::P1 => BOARD_HEIGHT - 2..BOARD_HEIGHT,
            PlayerID::P2 => 0..2,
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
    fn init(&mut self, id: PlayerID) -> Result<[Position; 4], Self::Error>;
    fn step(&mut self, board: Board) -> Result<Move, Self::Error>;
    fn close(&mut self, _victory: bool) {}
}
