use common::{
    misc::{DrawCause, Outcome, WinCause},
    Move, Player,
};

use crate::{FromBytes, ToBytes};

impl ToBytes for Move {
    fn to_bytes<W: std::io::Write>(&self, bytes: &mut W) -> std::io::Result<()> {
        bytes.write_all(&self.to_u16().to_le_bytes())
    }
}

impl FromBytes for Move {
    fn from_bytes(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, bytes) = nom::bytes::streaming::take(2usize)(b)?;
        let bytes: [u8; 2] = bytes.try_into().unwrap();
        Ok((i, Move::from_u16(u16::from_le_bytes(bytes))))
    }
}

impl ToBytes for Outcome {
    fn to_bytes<W: std::io::Write>(&self, bytes: &mut W) -> std::io::Result<()> {
        let byte = match self {
            Outcome::Won { by, cause } => {
                let byte = match cause {
                    WinCause::Timeout => 0b1100,
                    WinCause::Mate => 0b1011,
                    WinCause::Disconnect => 0b1010,
                    WinCause::Other => 0b1001,
                };
                if *by == Player::White {
                    byte | 0b10000
                } else {
                    byte
                }
            }
            Outcome::Drawn(cause) => match cause {
                DrawCause::Stalemate => 0b1000,
                DrawCause::Timeout => 0b0111,
                DrawCause::FiftyMove => 0b0110,
                DrawCause::Repetition => 0b0101,
                DrawCause::Agreement => 0b0100,
                DrawCause::Material => 0b0011,
                DrawCause::Disconnect => 0b0010,
                DrawCause::Other => 0b0001,
            },
            Outcome::None => 0,
        };
        bytes.write_all(&[byte])?;
        Ok(())
    }
}

impl FromBytes for Outcome {
    fn from_bytes(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, o) = nom::bytes::streaming::take(1usize)(b)?;
        let o = o[0];
        let v = match o & 0b1111 {
            0b1001..=0b1100 => {
                let by = if o & 0b10000 != 0 {
                    Player::Black
                } else {
                    Player::White
                };
                let cause = match o & 0b1111 {
                    0b1100 => WinCause::Timeout,
                    0b1011 => WinCause::Mate,
                    0b1010 => WinCause::Disconnect,
                    _ => WinCause::Other,
                };
                Outcome::Won { by, cause }
            }
            0b1000 => Outcome::Drawn(DrawCause::Stalemate),
            0b0111 => Outcome::Drawn(DrawCause::Timeout),
            0b0110 => Outcome::Drawn(DrawCause::FiftyMove),
            0b0101 => Outcome::Drawn(DrawCause::Repetition),
            0b0100 => Outcome::Drawn(DrawCause::Agreement),
            0b0011 => Outcome::Drawn(DrawCause::Material),
            0b0010 => Outcome::Drawn(DrawCause::Disconnect),
            0b0001 => Outcome::Drawn(DrawCause::Other),

            _ => Outcome::None,
        };
        Ok((i, v))
    }
}
