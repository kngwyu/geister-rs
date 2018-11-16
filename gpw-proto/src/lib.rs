use geister_core::{
    board::{Board, Cell, Direction, Ghost, GhostID, Move, Position},
    player::{Player, PlayerID},
};
use std::io::{self, prelude::*};
use std::net::{IpAddr, TcpStream};
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

pub trait GpwBoard: Sized {
    fn from_gpw<C>(s: &str, player: PlayerID) -> Result<Self, Error<C>>;
}

fn read1<T: str::FromStr, C>(s: &str, start: usize) -> Result<T, Error<C>>
where
    <T as str::FromStr>::Err: ::std::fmt::Debug,
{
    s[start..start + 1]
        .parse::<T>()
        .map_err(|_| Error::ParseError(s.to_owned()))
}

impl GpwBoard for Board {
    fn from_gpw<C>(s: &str, player: PlayerID) -> Result<Self, Error<C>> {
        let mut board = Board::default();
        for i in 0..8 {
            let start = 3 * i;
            let x: i8 = read1(s, start)?;
            let y: i8 = read1(s, start + 1)?;
            let ghost = match read1::<char, _>(s, start + 2)? {
                'R' => Ghost::Red,
                'B' => Ghost::Blue,
                _ => continue,
            };
            board[Position::new(x, y)] =
                Cell::owned(ghost, player, GhostID::from_u8(i as u8).unwrap());
        }
        for i in 0..8 {
            let start = 3 * (i + 7);
            let x: i8 = read1(s, start)?;
            let y: i8 = read1(s, start + 1)?;
            let ghost = match read1::<char, _>(s, start + 2)? {
                'u' => Ghost::Unknown,
                _ => continue,
            };
            board[Position::new(x, y)] =
                Cell::owned(ghost, player.rev(), GhostID::from_u8(i as u8).unwrap());
        }
        Ok(board)
    }
}

pub trait GpwPlayer: Player {
    fn gpw_step(&mut self, board: &str) -> Result<String, Error<Self::Error>> {
        let board = Board::from_gpw(board, self.id())?;
        let mov = self.step(board).map_err(Error::Agent)?;
        println!("{:?}", mov);
        let cell = self.board()[mov.pos];
        match cell {
            Cell::Owned(o) => Ok(mov.to_gpw(o.id())),
            Cell::Empty => Err(Error::InvalidMove(mov)),
        }
    }
}

impl<P: Player> GpwPlayer for P {}

#[derive(Debug)]
pub enum Error<C> {
    Agent(C),
    Io(io::Error),
    InvalidMove(Move),
    Mismatch(String),
    ParseError(String),
}

impl<C> From<io::Error> for Error<C> {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

fn read(tcp: &mut TcpStream) -> io::Result<String> {
    let mut res = String::new();
    let mut buf = vec![0u8; 1024];
    loop {
        let n = tcp.read(&mut buf)?;
        let s = ::std::str::from_utf8(&buf[..n]).unwrap();
        if let Some(pos) = s.find("\r\n") {
            res += &s[..pos];
            return Ok(res);
        } else {
            res += s;
        }
    }
}

fn expect<C>(tcp: &mut TcpStream, ok: impl Fn(&str) -> bool) -> Result<(), Error<C>> {
    let s = read(tcp)?;
    if !ok(&s) {
        return Err(Error::Mismatch(s));
    }
    Ok(())
}

fn expect_ok<C>(tcp: &mut TcpStream) -> Result<(), Error<C>> {
    expect(tcp, |s| s.starts_with("OK"))
}

fn port(id: PlayerID) -> u16 {
    match id {
        PlayerID::P1 => 10000,
        PlayerID::P2 => 10001            ,
    }
}

pub fn run_client<C: GpwPlayer>(client: &mut C, addr: IpAddr) -> Result<(), Error<C::Error>> {
    let id = client.id();
    let start_pos = client.init(id).map_err(Error::Agent)?;    
    let mut tcp = TcpStream::connect((addr, port(id)))?;
    expect(&mut tcp, |s| s == "SET?")?;
    let init_pos: Vec<_> = start_pos.iter().map(|x| x.init_pos().unwrap()).collect();
    write!(tcp, "SET:{}\r\n", str::from_utf8(&init_pos).unwrap())?;
    expect_ok(&mut tcp)?;
    loop {
        let board = read(&mut tcp)?;
        match &board[..4] {
            "MOV?" => {
                let mov = client.gpw_step(&board[4..])?;
                println!("{:?}", client.board());
                tcp.write(mov.as_bytes())?;
                expect_ok(&mut tcp)?;
            },
            "LST:" => {
                println!("LOSE (´・ω・`)");
                break;                
            },
            "WON:" => {
                println!("WIN (*´ω｀*)");
                break;
            },
            _ => return Err(Error::Mismatch(board.to_owned())),
        }
    }
    Ok(())
}
