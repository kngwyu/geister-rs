use crate::{board::*, player::*};
use crate::error::ErrorKind;

#[derive(Clone, )]
pub struct Simulator {
    board: Board,
    next: PlayerID,
}

pub enum Transition {
    Lost(OwnedCell),
    End(PlayerID),
    None,
}

impl Simulator {
    pub fn transit(&mut self, next: Board) -> Transition {
        let diffs = self.board.diff(&next);
        self.board = next;
        self.next = self.next.rev();
        for diff in diffs {
            let Diff {
                pos,
                before,
                after,
            } = diff;
            match before {
                Cell::Owned(before) => match after {
                    Cell::Owned(_) => return Transition::Lost(before),
                    Cell::Empty => {
                        let owner = before.owner();
                        if before.ghost() == Ghost::Blue && pos.is_escape(owner) {
                            return Transition::End(owner);
                        }
                    }
                }
                Cell::Empty => continue,
            }
        }
        Transition::None
    }
    pub fn play<F>(&mut self, policy: F) -> Transition
    where
        F: FnOnce(&Board, PlayerID) -> Move,
    {
        let mov = policy(&self.board, self.next);
        Transition::None
    }
    pub fn playout(start: PlayerID) -> Result<PlayerID, ErrorKind> {        
        Ok(start)
    }
}
