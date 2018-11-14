use crate::board::Position;

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum ErrorKind {
    #[display(fmt = "Invalid Position {}", _0)]
    IndexError(Position),
}
