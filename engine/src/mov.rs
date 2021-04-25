use super::Square;
use std::fmt;

#[derive(Clone,Copy,Eq,PartialEq)]
pub struct Move(u16);

impl Move{
    pub const FROM_MASK: u16 = 0b111_111;
    pub const TO_MASK: u16 = 0b111_111 << 6;

    pub const TYPE_MASK: u16 = 0b11 << 12;
    pub const TYPE_CASTLE: u16 = 1 << 12;
    pub const TYPE_EN_PASSANT: u16 = 2 << 12;
    pub const TYPE_PROMOTION: u16 = 3 << 12;
    pub const TYPE_NORMAL: u16 = 0;
    
    pub const PROMOTION_MASK: u16 = 0b11<< 14;

    pub const PROMOTION_QUEEN: u16 = 0 << 14;
    pub const PROMOTION_KNIGHT: u16 = 1 << 14;
    pub const PROMOTION_ROOK: u16 = 2 << 14;
    pub const PROMOTION_BISHOP: u16 = 3 << 14;

    pub fn new(from: Square, to: Square, ty: u16, promotion: u16) -> Self{
        Self(from.get() as u16 | (to.get() as u16) << 6| ty | promotion)
    }

    pub fn normal(from: Square, to: Square) -> Self{
        Self(from.get() as u16 | (to.get() as u16) << 6)
    }

    pub fn castle(from: Square, to: Square) -> Self{
        Self(from.get() as u16 | (to.get() as u16) << 6 | Self::TYPE_CASTLE)
    }

    pub fn promotion(from: Square, to: Square, promotion: u16) -> Self{
        debug_assert!(promotion >= Self::PROMOTION_QUEEN && promotion <= Self::PROMOTION_BISHOP);
        Self(from.get() as u16 | (to.get() as u16) << 6 | Self::TYPE_PROMOTION | promotion)
    }

    pub fn en_passant(from: Square, to: Square) -> Self{
        Self(from.get() as u16 | (to.get() as u16) << 6 | Self::TYPE_EN_PASSANT)
    }

    pub fn from(self) -> Square{
        Square::new((self.0 & Self::FROM_MASK) as u8)
    }
    
    pub fn to(self) -> Square{
        Square::new(((self.0 & Self::TO_MASK) >> 6) as u8)
    }

    pub fn ty(self) -> u16{
        self.0 & Self::TYPE_MASK
    }

    pub fn promotion_piece(self) -> u16{
        debug_assert_eq!(self.ty(), Self::TYPE_PROMOTION);
        self.0 & Self::PROMOTION_MASK
    }
}

impl fmt::Debug for Move{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("value",&self.0)
            .field("notation",&format!("{}",self))
            .field("ty",&(self.ty() >> 12))
            .field("to",&self.to())
            .field("from",&self.from())
            .field("promotion",&if self.ty() == Self::TYPE_PROMOTION{ 
                Some(self.promotion_piece() >> 14)
            }else{
                None
            })
            .finish()
    }
}

impl fmt::Display for Move{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ty() == Self::TYPE_CASTLE{
            if self.to() == Square::C1 || self.to() == Square::C8{
                return write!(f,"O-O({},{})",self.from(),self.to());
            }else{
                return write!(f,"O-O-O({},{})",self.from(),self.to());
            }
        }
        write!(f,"{}{}",self.from(),self.to())?;
        if self.ty() == Self::TYPE_PROMOTION{
            let piece = match self.promotion_piece(){
                Self::PROMOTION_QUEEN => "Q",
                Self::PROMOTION_KNIGHT => "K",
                Self::PROMOTION_ROOK => "R",
                Self::PROMOTION_BISHOP => "B",
                _ => unreachable!(),
            };
            return write!(f,"={}",piece);
        }
        if self.ty() == Self::TYPE_EN_PASSANT{
            return write!(f,"e.p.");
        }

        Ok(())
    }
}

/*
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Move {
    Quiet {
        from: Square,
        to: Square,
        piece: Piece,
    },
    Capture {
        from: Square,
        to: Square,
        piece: Piece,
        taken: Piece,
    },
    Promote {
        promote: Piece,
        to: Square,
        from: Square,
    },
    PromoteCapture {
        promote: Piece,
        taken: Piece,
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
        Piece::WhiteKnight => write!(f, "N"),
        Piece::BlackKnight => write!(f, "n"),
        Piece::WhitePawn | Piece::BlackPawn => Ok(()),
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Move::Quiet{ from, to, piece } => {
                write_piece(piece, f)?;
                write!(f, "{:?}{:?}", from, to)?;
            }
            Move::Capture{ from, to, piece,taken } => {
                write_piece(piece, f)?;
                write!(f, "{:?}{:?}x", from, to)?;
                write_piece(taken, f)?;
            }
            Move::Promote { to, from, promote } => {
                write!(f, "{:?}{:?}=", from, to)?;
                write_piece(promote, f)?;
            }
            Move::PromoteCapture { to, from, promote,taken } => {
                write!(f, "{:?}{:?}x", from, to)?;
                write_piece(taken, f)?;
                write!(f, "=")?;
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
*/
