use std::io::{self, BufRead, BufReader, Read};

use common::{misc::Outcome, Move};
use nom::{Err, Needed, Offset};

use crate::{board::FileBoard, FromBytes, ToBytes};

#[derive(Debug)]
pub struct Game {
    pub start_position: FileBoard,
    pub moves: Vec<Move>,
    pub outcome: Outcome,
}

pub struct GameReader<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> GameReader<R> {
    fn read_exceeds(&mut self, mut needed: Needed) -> Option<io::Result<Game>> {
        let mut buffer = self.reader.buffer().to_vec();
        self.reader.consume(buffer.len());
        match self.reader.fill_buf() {
            Err(e) => return Some(Err(e)),
            Ok(x) => {
                if x.is_empty() {
                    return None;
                }
            }
        };

        loop {
            let consume = match needed {
                Needed::Unknown => 1,
                Needed::Size(x) => x.into(),
            };
            let consume = self.reader.buffer().len().min(consume);
            buffer.extend_from_slice(&self.reader.buffer()[..consume]);
            self.reader.consume(consume);
            match Game::from_bytes(&buffer) {
                Ok((_, x)) => return Some(Ok(x)),
                Err(Err::Incomplete(n)) => {
                    needed = n;
                }
                _ => return None,
            }

            if self.reader.buffer().is_empty() {
                match self.reader.fill_buf() {
                    Err(e) => return Some(Err(e)),
                    Ok(x) => {
                        if x.is_empty() {
                            return None;
                        }
                    }
                };
            }
        }
    }
}

impl<R: Read> Iterator for GameReader<R> {
    type Item = io::Result<Game>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match Game::from_bytes(self.reader.buffer()) {
                Ok((i, x)) => {
                    let consumed = self.reader.buffer().offset(i);
                    self.reader.consume(consumed);
                    return Some(Ok(x));
                }
                Err(Err::Incomplete(needed)) => {
                    if self.reader.buffer().len() < self.reader.capacity() {
                        match self.reader.fill_buf() {
                            Err(e) => return Some(Err(e)),
                            Ok(x) => {
                                if x.is_empty() {
                                    return None;
                                }
                            }
                        }
                    } else {
                        return self.read_exceeds(needed);
                    }
                }
                _ => return None,
            }
        }
    }
}

impl ToBytes for Game {
    fn to_bytes<W: std::io::Write>(&self, bytes: &mut W) -> std::io::Result<()> {
        self.start_position.to_bytes(bytes)?;
        self.outcome.to_bytes(bytes)?;
        let len: u32 = self.moves.len().try_into().map_err(io::Error::other)?;
        let len_bytes = len.to_le_bytes();
        bytes.write_all(&len_bytes)?;
        for b in &self.moves {
            b.to_bytes(bytes)?
        }
        Ok(())
    }
}

impl FromBytes for Game {
    fn from_bytes(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let (b, start_position) = FileBoard::from_bytes(b)?;
        let (b, outcome) = Outcome::from_bytes(b)?;
        let (mut b, len) = nom::bytes::streaming::take(4usize)(b)?;
        let len = u32::from_le_bytes(len.try_into().unwrap());
        let mut moves = Vec::new();
        for _ in 0..len {
            let (t, m) = Move::from_bytes(b)?;
            b = t;
            moves.push(m);
        }
        Ok((
            b,
            Game {
                start_position,
                outcome,
                moves,
            },
        ))
    }
}
