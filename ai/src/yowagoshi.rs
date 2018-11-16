use geister_core::{board::*, player::{PlayerID, Player as PlayerT}};
use crate::rnghandle::RngHandle;
use crate::metadata::MetaDataList;
use crate::random;


#[derive(Clone, Copy, Debug, Default, Add, Sub, AddAssign, SubAssign, Eq, PartialEq, PartialOrd, Ord, Mul)]
struct Eval(i64);

impl Eval {
    fn lost() -> Self {
        Eval(-2000)
    }
    fn around(ghost: Ghost) -> Self {
        match ghost {
            Ghost::Blue => Eval(-50),
            Ghost::Red => Eval(50),
            _ => Eval(0),
        }
    }
    fn phase(player: PlayerID, pos: Position) -> Self {
        match player {
            PlayerID::P1 => {
                if pos.y <= 2 {
                    Eval(-50)
                } else {
                    Eval(50)
                }
            }
            PlayerID::P2 => {
                if pos.y >= 3 {
                    Eval(50)
                } else {
                    Eval(50)
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    id: PlayerID,
    board: Board,
    rng: RngHandle,
    meta: MetaDataList,
}

impl Player {
    pub fn new(id: PlayerID) -> Self {
        Player {
            id,
            board: Board::default(),
            rng: RngHandle::default(),
            meta: MetaDataList::new(),
        }
    }
}

#[derive(Clone, Debug, Display)]
pub struct SearchError(String);

impl SearchError {
    fn from_str(s: &str) -> Self {
        SearchError(s.to_owned())
    }
}

fn unimpl() -> SearchError {
    SearchError::from_str("Unimplemented!")
}

impl PlayerT for Player {
    type Error = SearchError;
    fn id(&self) -> PlayerID {
        self.id
    }
    fn init(&mut self, id: PlayerID) -> Result<[Position; 4], Self::Error> {
        assert_eq!(id, self.id);
        self.id = id;
        Ok(random::random_init(id, &mut self.rng))
    }
    fn board(&self) -> &Board {
        &self.board
    }
    fn step(&mut self, board: Board) -> Result<Move, Self::Error> {
        let mut cand = vec![];
        let diff = self.board.diff(&board);
        let _transition: Vec<_> = diff.into_iter().map(|x| x.into_transition()).collect();
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
        let mut evals = vec![];
        for mov in cand {
            let mut board = self.board.clone();
            let t = match board.transit(mov) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let to = mov.to();
            if to.is_escape(self.id.rev()) {
                return Ok(mov);
            }
            let mut eval = Eval::default();
            if let Transition::Lost(_) = t {
                eval += Eval::lost();
            }
            for d in Direction::iter() {
                let pos = to + d.to_pos();
                if pos.is_valid() {
                    if let Cell::Owned(o) = board[pos] {
                        if o.owner() == self.id.rev() {
                            eval.0 += Eval::around(o.ghost()).0 * Eval::phase(o.owner(), pos).0;
                        }
                    }
                }
            }
            evals.push((mov, eval));
        }
        evals.sort_by_key(|e| e.1);
        Ok(evals.last().unwrap().0)
    }
}

