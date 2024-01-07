use common::{board::Board, BoardArray, ExtraState, Player, Square, SquareContent};

use crate::{FromBytes, ToBytes};

#[derive(Debug)]
pub struct FileBoard {
    // squares 4 bits per square.
    squares: [u8; 32],
    // 1-4: en-passant
    // 5-8: castle-state
    en_passant_castle: u8,
    player_move_clock: u8,
}

impl FileBoard {
    pub fn from_board(board: &Board) -> Self {
        let mut squares = [0u8; 32];

        for s in 0..32 {
            let s1 = Square::new(s * 2);
            let s2 = Square::new(s * 2 + 1);
            let a = board.squares[s1] as u8;
            let b = board.squares[s2] as u8;
            squares[s as usize] = (a << 4) | b
        }

        let en_passant_castle = board.state.castle << 4 | board.state.en_passant;

        let mut player_move_clock = board.state.move_clock;
        if board.state.player == Player::Black {
            player_move_clock |= 128;
        }

        FileBoard {
            squares,
            en_passant_castle,
            player_move_clock,
        }
    }

    pub fn to_board(&self) -> Board {
        let state = ExtraState {
            player: if (self.player_move_clock & 128) == 0 {
                Player::White
            } else {
                Player::Black
            },
            move_clock: self.player_move_clock & 127,
            en_passant: self.en_passant_castle & 15,
            castle: self.en_passant_castle >> 4,
        };

        let mut squares = BoardArray::new(SquareContent::Empty);

        for s in 0..32 {
            let s1 = Square::new(s * 2);
            let s2 = Square::new(s * 2 + 1);

            let a = self.squares[s as usize] >> 4;
            let b = self.squares[s as usize] & 15;

            squares[s1] = Self::content_from_bits(a);
            squares[s2] = Self::content_from_bits(b);
        }

        Board::from_squares_state(squares, state)
    }

    fn content_from_bits(b: u8) -> SquareContent {
        match b {
            0b0000 => SquareContent::WhiteKing,
            0b0001 => SquareContent::BlackKing,

            0b0010 => SquareContent::WhitePawn,
            0b0011 => SquareContent::BlackPawn,

            0b0100 => SquareContent::WhiteBishop,
            0b0101 => SquareContent::BlackBishop,

            0b0110 => SquareContent::WhiteKnight,
            0b0111 => SquareContent::BlackKnight,

            0b1000 => SquareContent::WhiteRook,
            0b1001 => SquareContent::BlackRook,

            0b1010 => SquareContent::WhiteQueen,
            0b1011 => SquareContent::BlackQueen,
            _ => SquareContent::Empty,
        }
    }
}

impl ToBytes for FileBoard {
    fn to_bytes<W: std::io::Write>(&self, bytes: &mut W) -> std::io::Result<()> {
        bytes.write_all(&self.squares)?;
        bytes.write_all(&[self.en_passant_castle, self.player_move_clock])
    }
}

impl FromBytes for FileBoard {
    fn from_bytes(b: &[u8]) -> nom::IResult<&[u8], Self> {
        let (i, bytes) = nom::bytes::streaming::take(34usize)(b)?;
        let squares = bytes[..32].try_into().unwrap();
        Ok((
            i,
            Self {
                squares,
                en_passant_castle: bytes[32],
                player_move_clock: bytes[33],
            },
        ))
    }
}
