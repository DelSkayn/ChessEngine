use nom::{Err, Needed, Offset};
use std::{
    io::{self, BufRead, BufReader, Read},
    marker::PhantomData,
};

use crate::FromBytes;

pub struct FromBytesReader<T, R: Read> {
    reader: BufReader<R>,
    buffer: Vec<u8>,
    marker: PhantomData<T>,
}

impl<T: FromBytes, R: Read> FromBytesReader<T, R> {
    pub fn new(r: R) -> Self {
        FromBytesReader {
            reader: BufReader::new(r),
            buffer: Vec::new(),
            marker: PhantomData,
        }
    }

    fn read_exceeds(&mut self, mut needed: Needed) -> Option<io::Result<T>> {
        self.buffer.extend_from_slice(self.reader.buffer());
        self.reader.consume(self.buffer.len());
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
            self.buffer
                .extend_from_slice(&self.reader.buffer()[..consume]);
            self.reader.consume(consume);
            match T::from_bytes(&self.buffer) {
                Ok((_, x)) => {
                    self.buffer.clear();
                    return Some(Ok(x));
                }
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

impl<T: FromBytes + std::fmt::Debug, R: Read> Iterator for FromBytesReader<T, R> {
    type Item = io::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match T::from_bytes(self.reader.buffer()) {
            Ok((i, x)) => {
                let consumed = self.reader.buffer().offset(i);
                self.reader.consume(consumed);
                Some(Ok(x))
            }
            Err(Err::Incomplete(needed)) => self.read_exceeds(needed),
            _ => None,
        }
    }
}
