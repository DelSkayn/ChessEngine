use std::io::{self, Write};

mod board;
mod game;
mod r#impl;
mod reader;

pub use board::FileBoard;
pub use game::Game;
pub use reader::FromBytesReader;

pub trait ToBytes {
    fn to_bytes<W: Write>(&self, bytes: &mut W) -> io::Result<()>;
}

pub trait FromBytes: Sized {
    fn from_bytes(b: &[u8]) -> nom::IResult<&[u8], Self>;
}
