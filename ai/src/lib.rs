#![feature(never_type)]

use clap;
use geister_core::{board::*, player::*};
mod rnghandle;
use self::rnghandle::RngHandle;

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

impl Player for RandomAi {
    type Error = ();
    fn id(&self) -> PlayerID {
        self.id
    }
    fn init(&mut self, id: PlayerID) -> Result<[Position; 4], ()> {
        self.id = id;
        let init_pos = id.init_pos();
        let mut res = [Position::new(0, 0); 4];
        for (i, idx) in self.rng.select(0..8).take(4).enumerate() {
            res[i] = init_pos[idx];
        }
        Ok(res)
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
                if self.board.can_move(mov).unwrap_or(false) {
                    cand.push(mov);
                }
            }
        }
        let idx = self.rng.select(0..cand.len()).next().unwrap();
        Ok(cand[idx])
    }
}

pub fn args<'a, 'b>(app: clap::App<'a, 'b>) -> clap::ArgMatches<'a> {
    app.arg(
        clap::Arg::with_name("id")
            .short("i")
            .long("id")
            .value_name("ID")
            .help("Player ID")
            .required(true)
            .takes_value(true),
    )
    .subcommand(
        clap::SubCommand::with_name("random")
            .about("random player")
            .version("0.1"),
    )
    .get_matches()
}
