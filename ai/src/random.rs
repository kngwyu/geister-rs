use geister_core::{board::*, player::*};
use crate::rnghandle::RngHandle;

pub struct RandomAi {
    id: PlayerID,
    board: Board,
    rng: RngHandle,
}

impl RandomAi {
    pub fn new(id: PlayerID) -> Self {
        RandomAi {
            id,
            board: Board::default(),
            rng: RngHandle::default(),
        }
    }
}

pub fn random_init(id: PlayerID, rng: &mut RngHandle) -> [Position; 4] {
    let init_pos = id.init_pos();
    let mut res = [Position::new(0, 0); 4];
    for (i, idx) in rng.select(0..8).take(4).enumerate() {
        res[i] = init_pos[idx];
    }
    res
}

impl Player for RandomAi {
    type Error = ();
    fn id(&self) -> PlayerID {
        self.id
    }
    fn init(&mut self, id: PlayerID) -> Result<[Position; 4], ()> {
        self.id = id;
        Ok(random_init(id, &mut self.rng))
    }
    fn board(&self) -> &Board {
        &self.board
    }
    fn step(&mut self, board: Board) -> Result<Move, ()> {
        let mut cand = vec![];
        self.board = board;
        for (x, y) in Board::iter() {
            let pos = Position::new(x, y);
            if self.board[pos].owner().unwrap_or(self.id.rev()) != self.id {
                continue;
            }
            for d in Direction::iter() {
                let mov = Move { pos, direction: d };
                match self.board.can_move(mov) {
                    MoveResult::Err => continue,
                    MoveResult::Ok => cand.push(mov),
                    MoveResult::Win => return Ok(mov),
                }
            }
        }
        let idx = self.rng.range(0..cand.len());
        Ok(cand[idx])
    }
}
