use common::{Move, Promotion, Square};
use nom::{
    branch::alt,
    character::complete::{char, satisfy},
    combinator::{opt, value},
    IResult,
};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct UciMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Promotion>,
}

impl UciMove {
    pub fn parse_partial(i: &str) -> IResult<&str, UciMove> {
        r#move(i)
    }

    pub fn to_move(&self, possible_moves: &[Move]) -> Option<Move> {
        possible_moves
            .iter()
            .find(|x| {
                x.to() == self.to && x.from() == self.from && x.get_promotion() == self.promotion
            })
            .copied()
    }
}

impl From<Move> for UciMove {
    fn from(value: Move) -> Self {
        UciMove {
            from: value.from(),
            to: value.to(),
            promotion: value.get_promotion(),
        }
    }
}

impl fmt::Display for UciMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(x) = self.promotion {
            match x {
                Promotion::Queen => write!(f, "q")?,
                Promotion::Knight => write!(f, "k")?,
                Promotion::Rook => write!(f, "r")?,
                Promotion::Bishop => write!(f, "b")?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UciMoveError;

impl fmt::Display for UciMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to parse UCI move")
    }
}

impl FromStr for UciMove {
    type Err = UciMoveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok((_, this)) = Self::parse_partial(s) else {
            return Err(UciMoveError);
        };
        Ok(this)
    }
}

fn file(i: &str) -> IResult<&str, u8> {
    let (i, r) = satisfy(|x| ('a'..='h').contains(&x))(i)?;
    Ok((i, r as u8 - b'a'))
}

fn rank(i: &str) -> IResult<&str, u8> {
    let (i, r) = satisfy(|x| ('1'..='8').contains(&x))(i)?;
    Ok((i, r as u8 - b'1'))
}

fn r#move(i: &str) -> IResult<&str, UciMove> {
    let (i, f) = file(i)?;
    let (i, r) = rank(i)?;
    let from = Square::from_file_rank(f, r);
    let (i, f) = file(i)?;
    let (i, r) = rank(i)?;
    let to = Square::from_file_rank(f, r);

    let (i, promotion) = opt(alt((
        value(Promotion::Queen, char('q')),
        value(Promotion::Rook, char('r')),
        value(Promotion::Bishop, char('b')),
        value(Promotion::Knight, char('k')),
    )))(i)?;

    Ok((
        i,
        UciMove {
            from,
            to,
            promotion,
        },
    ))
}
