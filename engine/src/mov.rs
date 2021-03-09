use super::{Piece, Square};
use std::fmt;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Move {
    Simple {
        from: Square,
        to: Square,
        piece: Piece,
    },
    Promote {
        promote: Piece,
        to: Square,
        from: Square,
    },
    Castle {
        king: bool,
    },
    EnPassant {
        to: Square,
        from: Square,
    },
}

fn write_piece(p: Piece, f: &mut fmt::Formatter) -> fmt::Result {
    match p {
        Piece::WhiteKing => write!(f, "K"),
        Piece::BlackKing => write!(f, "k"),
        Piece::WhiteQueen => write!(f, "Q"),
        Piece::BlackQueen => write!(f, "q"),
        Piece::WhiteRook => write!(f, "R"),
        Piece::BlackRook => write!(f, "r"),
        Piece::WhiteBishop => write!(f, "B"),
        Piece::BlackBishop => write!(f, "b"),
        Piece::WhiteKnight => write!(f, "K"),
        Piece::BlackKnight => write!(f, "k"),
        Piece::WhitePawn | Piece::BlackPawn => Ok(()),
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Move::Simple { from, to, piece } => {
                write_piece(piece, f)?;
                write!(f, "{:?}{:?}", from, to)?;
            }
            Move::Promote { to, from, promote } => {
                write!(f, "{:?}{:?}=", from, to)?;
                write_piece(promote, f)?;
            }
            Move::Castle { king } => {
                if king {
                    write!(f, "O-O")?;
                } else {
                    write!(f, "O-O-O")?;
                }
            }
            Move::EnPassant { to, from } => {
                write!(f, "{:?}x{:?}", from, to)?;
            }
        }
        Ok(())
    }
}
