use crate::board::Move;

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum ErrorKind {
    #[display(fmt = "Invalid Move {:?}", _0)]
    InvalidMove(Move),
}

impl<T> Into<Result<T, ErrorKind>> for ErrorKind {
    fn into(self) -> Result<T, ErrorKind> {
        Err(self)
    }
}
