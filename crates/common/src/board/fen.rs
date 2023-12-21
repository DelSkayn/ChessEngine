use core::fmt::{self, Write};
use std::str::FromStr;

use crate::{ExtraState, Piece, Player, Square, BB};

use super::Board;

#[derive(Debug, Clone, Copy)]
pub struct FenError;

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse FEN string")
    }
}

impl Board {
    pub fn from_fen_partial(fen: &str) -> Result<(Self, &str), FenError> {
        let mut board = Board::empty();

        let mut iter = fen.chars();

        let mut file = 0u8;
        let mut rank = 0u8;

        loop {
            let next = iter.next().ok_or(FenError)?;
            match next {
                x if x.is_ascii_digit() => {
                    file = file.saturating_add(x as u8 - b'0');
                }
                '/' => {
                    if file != 8 {
                        return Err(FenError);
                    }
                    file = 0;
                    rank += 1
                }
                ' ' => {
                    if file != 8 || rank != 7 {
                        return Err(FenError);
                    }
                    break;
                }
                x => {
                    if file > 7 || rank > 7 {
                        return Err(FenError);
                    }

                    let p = Piece::from_char(x).ok_or(FenError)?;
                    let s = Square::from_file_rank(file, 7 - rank);
                    file += 1;
                    board.squares[s] = p.into();
                    board.pieces[p] |= BB::square(s);
                }
            }
        }

        match iter.next().ok_or(FenError)? {
            'b' => board.state.player = Player::Black,
            'w' => {}
            _ => return Err(FenError),
        }

        if iter.next().ok_or(FenError)? != ' ' {
            return Err(FenError);
        }

        board.state.castle = 0;
        let mut next = iter.next().ok_or(FenError)?;
        if next != '-' {
            if next == 'K' {
                board.state.castle |= ExtraState::WHITE_KING_CASTLE;
                next = iter.next().ok_or(FenError)?;
            }

            if next == 'Q' {
                board.state.castle |= ExtraState::WHITE_QUEEN_CASTLE;
                next = iter.next().ok_or(FenError)?;
            }

            if next == 'k' {
                board.state.castle |= ExtraState::BLACK_KING_CASTLE;
                next = iter.next().ok_or(FenError)?;
            }

            if next == 'q' {
                board.state.castle |= ExtraState::BLACK_QUEEN_CASTLE;
                next = iter.next().ok_or(FenError)?;
            }

            if board.state.castle == 0 || next != ' ' {
                return Err(FenError);
            }
        } else if iter.next().ok_or(FenError)? != ' ' {
            return Err(FenError);
        }

        next = iter.next().ok_or(FenError)?;

        if next != '-' {
            if !('a'..='h').contains(&next) {
                return Err(FenError);
            }
            let file = b'a' - next as u8;
            next = iter.next().ok_or(FenError)?;
            if next != '3' && next != '6' {
                return Err(FenError);
            }
            board.state.en_passant = file;
        } else {
            board.state.en_passant = 8;
        }

        let before = iter.as_str();
        let Some(next) = iter.next() else {
            return Ok((board, before));
        };

        if next != ' ' {
            return Ok((board, before));
        }

        let Some(next) = iter.next() else {
            return Ok((board, iter.as_str()));
        };

        if !next.is_ascii_digit() {
            return Ok((board, before));
        }

        board.state.move_clock = next as u8 - b'0';

        let before = iter.as_str();
        let Some(next) = iter.next() else {
            return Ok((board, before));
        };

        if next.is_ascii_digit() {
            board.state.move_clock *= 10;
            board.state.move_clock = next as u8 - b'0';
        } else if next != ' ' {
            return Ok((board, before));
        }

        let before = iter.as_str();
        let Some(next) = iter.next() else {
            return Ok((board, before));
        };

        if next != ' ' {
            return Ok((board, before));
        }

        let Some(next) = iter.next() else {
            return Ok((board, before));
        };

        if !next.is_ascii_digit() {
            return Ok((board, before));
        }

        let before = iter.as_str();
        let Some(next) = iter.next() else {
            return Ok((board, before));
        };

        if !next.is_ascii_digit() {
            return Ok((board, before));
        }

        Ok((board, iter.as_str()))
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        Self::from_fen_partial(fen).map(|x| x.0)
    }

    pub fn to_fen(&self) -> String {
        let mut buffer = String::new();

        let mut accum = 0;
        for r in 0..8 {
            for f in 0..8 {
                let r = 7 - r;
                let s = Square::from_file_rank(f, r);
                if let Some(x) = self.squares[s].to_piece() {
                    if accum > 0 {
                        buffer.push(char::from_digit(accum, 10).unwrap());
                        accum = 0;
                    }
                    buffer.push(x.to_char());
                } else {
                    accum += 1;
                }
            }
            if accum > 0 {
                buffer.push(char::from_digit(accum, 10).unwrap());
                accum = 0;
            }
            if r != 7 {
                buffer.push('/');
            }
        }

        buffer.push(' ');
        if self.state.player == Player::White {
            buffer.push('w');
        } else {
            buffer.push('b');
        }

        buffer.push(' ');
        if self.state.castle & ExtraState::WHITE_KING_CASTLE != 0 {
            buffer.push('K');
        }
        if self.state.castle & ExtraState::WHITE_QUEEN_CASTLE != 0 {
            buffer.push('Q');
        }
        if self.state.castle & ExtraState::BLACK_KING_CASTLE != 0 {
            buffer.push('k');
        }
        if self.state.castle & ExtraState::BLACK_QUEEN_CASTLE != 0 {
            buffer.push('q');
        }
        if self.state.castle == 0 {
            buffer.push('-');
        }
        buffer.push(' ');
        if self.state.en_passant == 8 {
            buffer.push('-');
        } else {
            buffer.push((b'a' + self.state.en_passant) as char);
            if self.state.player == Player::White {
                buffer.push('6');
            } else {
                buffer.push('3');
            }
        }
        buffer.push(' ');
        write!(&mut buffer, "{}", self.state.move_clock).unwrap();
        buffer.push(' ');
        // just use move clock since we don't keep track of the number of moves made.
        write!(&mut buffer, "{}", self.state.move_clock as u32 + 1).unwrap();

        buffer
    }
}

impl FromStr for Board {
    type Err = FenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Board::from_fen(s)
    }
}
