use guister_core::{
    board::{Board, Cell, Ghost, Move, Position, GhostID, Direction},
    player::{Player, PlayerID},
};
use std::io::{self, prelude::*};
use std::net::{Ipv4Addr, TcpStream};
use std::str;

pub trait GpwPosition {
    fn init_pos(&self) -> Option<u8>;
}

impl GpwPosition for Position {
    fn init_pos(&self) -> Option<u8> {
        let Position { x, y } = *self;
        let offset = match x {
            x if 1 <= x && x <= 4 => x as u8 - 1,
            _ => return None,
        };
        Some(match y {
            0 => b'h' - offset,
            1 => b'd' - offset,
            4 => b'A' + offset,
            5 => b'E' + offset,
            _ => return None,
        })
    }
}

pub trait GpwMove: Sized {
    fn to_gpw(self, id: GhostID) -> String;    
}

impl GpwMove for Move {
    fn to_gpw(self, id: GhostID) -> String {
        let d = match self.direction {
            Direction::Up => 'N',
            Direction::Down => 'S',
            Direction::Left => 'W',
            Direction::Right => 'E',
        };
        format!("MOV:{},{}\r\n", id, d)
    }
}

pub trait Gpwboard: Sized {
    fn from_gpw<C>(s: &str, player: PlayerID) -> Result<Self, Error<C>>;
}

fn read1<T: str::FromStr, C>(s: &str, start: usize) -> Result<T, Error<C>> {
    s[start..start + 1]
        .parse::<T>()
        .map_err(|_| Error::ParseError(s.to_owned()))
}

impl Gpwboard for Board {
    fn from_gpw<C>(s: &str, player: PlayerID) -> Result<Self, Error<C>> {
        let mut board = Board::default();
        for i in 0..8 {
            let start = 3 * i;
            let x: i8 = read1(s, start)?;
            let y: i8 = read1(s, start + 1)?;
            let ghost = match read1::<u8, _>(s, start + 2)? {
                b'R' => Ghost::Red,
                b'B' => Ghost::Blue,
                _ => continue,
            };
            board[Position::new(x, y)] = Cell::owned(ghost, player, GhostID::from_u8(i as u8).unwrap());
        }
        for i in 0..8 {
            let start = 3 * i;
            let x: i8 = read1(s, start)?;
            let y: i8 = read1(s, start + 1)?;
            let ghost = match read1::<u8, _>(s, start + 2)? {
                b'u' => Ghost::Unknown,
                _ => continue,
            };
            board[Position::new(x, y)] = Cell::owned(ghost, player.rev(), GhostID::from_u8(i as u8).unwrap());
        }
        Ok(board)
    }
}

pub trait GpwPlayer: Player {
    fn gpw_step(&mut self, board: String) -> Result<String, Error<Self::Error>> {
        let board = Board::from_gpw(&board, self.id())?;
        let mov = self.step(board).map_err(Error::Agent)?;
        let cell = self.board()[mov.pos];
        match cell {
            Cell::Owned(o) => Ok(mov.to_gpw(o.id())),
            Cell::Empty => Err(Error::InvalidMove(mov)),
        }
    }
}

impl<P: Player> GpwPlayer for P {}

fn addr_from_id(id: PlayerID) -> (Ipv4Addr, u16) {
    match id {
        PlayerID::P1 => (Ipv4Addr::new(127, 0, 0, 1), 10000),
        PlayerID::P2 => (Ipv4Addr::new(127, 0, 0, 1), 10001),
    }
}

pub enum Error<C> {
    Agent(C),
    Io(io::Error),
    InvalidMove(Move),
    Mismatch,
    ParseError(String),
}

impl<C> From<io::Error> for Error<C> {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

fn expect<C>(tcp: &mut TcpStream, expect: &str) -> Result<(), Error<C>> {
    expect_fn(tcp, |s| s == expect)
}

fn expect_fn<C>(tcp: &mut TcpStream, ok: impl Fn(&str) -> bool) -> Result<(), Error<C>> {
    let mut buf = String::new();
    tcp.read_to_string(&mut buf)?;
    if !ok(&buf) {
        return Err(Error::Mismatch);
    }
    Ok(())
}

fn recv<C>(tcp: &mut TcpStream) -> Result<String, Error<C>> {
    let mut buf = String::new();
    tcp.read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn run_client<C: GpwPlayer>(client: &mut C, id: PlayerID) -> Result<(), Error<C::Error>> {
    let start_pos = client.init(id).map_err(Error::Agent)?;
    let mut tcp = TcpStream::connect(addr_from_id(id))?;
    expect(&mut tcp, "SET?")?;
    let init_pos: Vec<_> = start_pos.iter().map(|x| x.init_pos().unwrap()).collect();
    write!(tcp, "SET:{}\r\n", str::from_utf8(&init_pos).unwrap())?;
    expect(&mut tcp, "Ok\n\n")?;
    loop {
        let board = recv(&mut tcp)?;
        let mov = client.gpw_step(board)?;
        tcp.write(mov.as_bytes())?;
        expect_fn(&mut tcp, |s| match s {
            "OK\r\n"| "OKR\r\n" | "OKB\r\n" => true,
            _ => false,
        })?;
    }
    Ok(())
}
