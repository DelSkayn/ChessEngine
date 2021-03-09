use super::*;
use anyhow::{anyhow, bail, ensure, Result};

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self> {
        let mut board = Board::empty();

        let mut column = 0;
        let mut row = 0;
        let mut iterator = fen.chars();

        for c in &mut iterator {
            if let Some(off) = c.to_digit(9) {
                column += off;
                ensure!(column <= 8, "notation moved outside board");
                continue;
            }
            let bitmap = match c {
                '/' => {
                    row += 1;
                    ensure!(row <= 8, "notation moved outside board");
                    ensure!(column <= 8, "notation moved to far in column");
                    ensure!(column == 8, "notation did not use full row");
                    column = 0;
                    continue;
                }
                ' ' => {
                    break;
                }
                'K' => {
                    ensure!(
                        board[Piece::WhiteKing].none(),
                        "notation had multiple white kings!"
                    );
                    Piece::WhiteKing
                }
                'k' => {
                    ensure!(
                        board[Piece::BlackKing].none(),
                        "notation had multiple white kings!"
                    );
                    Piece::BlackKing
                }
                'Q' => Piece::WhiteQueen,
                'q' => Piece::BlackQueen,
                'R' => Piece::WhiteRook,
                'r' => Piece::BlackRook,
                'B' => Piece::WhiteBishop,
                'b' => Piece::BlackBishop,
                'N' => Piece::WhiteKnight,
                'n' => Piece::BlackKnight,
                'P' => Piece::WhitePawn,
                'p' => Piece::BlackPawn,
                x => {
                    bail!("invalid character: {}", x);
                }
            };
            ensure!(
                column <= 7,
                "notation tried to place piece outside the board"
            );
            board[bitmap] |= 1 << (7 - row) * 8 + column;
            column += 1;
        }

        if let Some(x) = iterator.next() {
            match x {
                'w' => {}
                'b' => board.state |= ExtraState::BLACK_MOVE,
                x => bail!("invalid character '{}', expected on of 'w', 'b'", x),
            }
        } else {
            bail!("missing characters!")
        }

        ensure!(
            iterator.next() == Some(' '),
            "missing space after current move indicator"
        );

        let mut cnt = 0;
        loop {
            match iterator.next() {
                Some('-') => {
                    ensure!(
                        cnt == 0,
                        "invalid character '-', expected one of 'K','Q','k','q',' '"
                    );
                    iterator.next();
                    break;
                }
                Some(' ') => {
                    ensure!(
                        cnt != 0,
                        "invalid character ' ', expected one of 'K','Q','k','q','-'"
                    );
                    break;
                }
                Some('K') => {
                    ensure!(cnt == 0, "invalid order castle rights");
                    cnt = 1;
                    board.state |= ExtraState::WHITE_KING_CASTLE;
                }
                Some('Q') => {
                    ensure!(cnt <= 1, "invalid order castle rights");
                    cnt = 2;
                    board.state |= ExtraState::WHITE_QUEEN_CASTLE;
                }
                Some('k') => {
                    ensure!(cnt <= 2, "invalid order castle rights");
                    cnt = 3;
                    board.state |= ExtraState::BLACK_KING_CASTLE;
                }
                Some('q') => {
                    cnt = 4;
                    board.state |= ExtraState::BLACK_QUEEN_CASTLE;
                }
                Some(x) => {
                    bail!("invalid character '{}'", x);
                }
                None => {
                    bail!("missing characters")
                }
            }
        }

        match iterator.next() {
            Some('-') => {}
            Some(c) => {
                if let Some(r) = iterator.next() {
                    let idx = Self::postion_to_square(c, r).ok_or(anyhow!("invalid position"))?;
                    board.state = board.state.set_en_passant(idx);
                } else {
                    bail!("missing characters")
                }
            }
            None => {
                bail!("missing characters")
            }
        }

        Ok(board)
    }

    fn postion_to_square(column: char, row: char) -> Option<Square> {
        if 'a' > column || 'h' < column {
            return None;
        }

        if '1' > row || '8' < row {
            return None;
        }

        let row = row as u8 - b'1';
        let column = column as u8 - b'a';

        return Some(Square::new(row * 8 + column));
    }
}
