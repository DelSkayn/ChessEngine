use super::{Board, MoveChain};
use crate::{ExtraState, Piece, Player, Square};
use anyhow::{anyhow, bail, ensure, Result};

impl<C: MoveChain> Board<C> {
    /// Create a board position from a fen string.
    /// Board's hash value has not been initialized
    pub fn from_fen(fen: &str, mut chain: C) -> Result<Self> {
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
                        board.pieces[Piece::WhiteKing].none(),
                        "notation had multiple white kings!"
                    );
                    Piece::WhiteKing
                }
                'k' => {
                    ensure!(
                        board.pieces[Piece::BlackKing].none(),
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
            board.pieces[bitmap] |= 1 << (7 - row) * 8 + column;
            column += 1;
        }

        if let Some(x) = iterator.next() {
            match x {
                'w' => {}
                'b' => board.state.player = Player::Black,
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
                    board.state.castle |= ExtraState::WHITE_KING_CASTLE;
                }
                Some('Q') => {
                    ensure!(cnt <= 1, "invalid order castle rights");
                    cnt = 2;
                    board.state.castle |= ExtraState::WHITE_QUEEN_CASTLE;
                }
                Some('k') => {
                    ensure!(cnt <= 2, "invalid order castle rights");
                    cnt = 3;
                    board.state.castle |= ExtraState::BLACK_KING_CASTLE;
                }
                Some('q') => {
                    cnt = 4;
                    board.state.castle |= ExtraState::BLACK_QUEEN_CASTLE;
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
                    let idx = Self::postion_to_square(c, r)
                        .ok_or(anyhow!("invalid position: invalid en passant square"))?;
                    board.state.en_passant = idx.file();
                } else {
                    bail!("missing characters")
                }
            }
            None => {
                bail!("missing characters")
            }
        }

        //ensure!(iterator.next() == Some('-'), "invalid position");
        let mut iterator = iterator.as_str().split_whitespace();

        let half_time = iterator
            .next()
            .ok_or_else(|| anyhow!("invalid position"))
            .and_then(|x| Ok(x.parse::<u8>()?))?;

        board.state.move_clock = half_time;

        let _move_time = iterator
            .next()
            .ok_or_else(|| anyhow!("invalid position"))
            .and_then(|x| Ok(x.parse::<u32>()?))?;

        for p in Piece::WhiteKing.to(Piece::BlackPawn) {
            for s in board.pieces[p].iter() {
                board.squares[s] = Some(p);
            }
        }

        chain.position(&board.pieces, board.state);

        Ok(Board {
            pieces: board.pieces,
            state: board.state,
            squares: board.squares,
            chain,
        })
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

    /// Returns the fen string of the board
    pub fn to_fen(&self) -> String {
        let mut res = String::new();

        for rank in 0..8 {
            let mut count = 0;
            let rank = 7 - rank;
            for file in 0..8 {
                if let Some(x) = self.on(Square::from_file_rank(file, rank)) {
                    if count > 0 {
                        res.push_str(&format!("{}", count));
                        count = 0;
                    }
                    match x {
                        Piece::WhiteKing => res.push('K'),
                        Piece::BlackKing => res.push('k'),
                        Piece::WhiteQueen => res.push('Q'),
                        Piece::BlackQueen => res.push('q'),
                        Piece::WhiteRook => res.push('R'),
                        Piece::BlackRook => res.push('r'),
                        Piece::WhiteBishop => res.push('B'),
                        Piece::BlackBishop => res.push('b'),
                        Piece::WhiteKnight => res.push('N'),
                        Piece::BlackKnight => res.push('n'),
                        Piece::WhitePawn => res.push('P'),
                        Piece::BlackPawn => res.push('p'),
                    };
                } else {
                    count += 1;
                }
            }
            if count > 0 {
                res.push_str(&format!("{}", count));
            }
            if rank != 0 {
                res.push('/');
            }
        }
        res.push(' ');
        match self.state.player {
            Player::White => {
                res.push('w');
            }
            Player::Black => {
                res.push('b');
            }
        }
        res.push(' ');
        let len = res.len();
        if self.state.castle & ExtraState::WHITE_KING_CASTLE != 0 {
            res.push('K');
        }
        if self.state.castle & ExtraState::WHITE_QUEEN_CASTLE != 0 {
            res.push('Q');
        }
        if self.state.castle & ExtraState::BLACK_KING_CASTLE != 0 {
            res.push('k');
        }
        if self.state.castle & ExtraState::BLACK_QUEEN_CASTLE != 0 {
            res.push('q');
        }
        if len == res.len() {
            res.push('-');
        }
        res.push(' ');
        if self.state.en_passant != ExtraState::INVALID_ENPASSANT {
            match self.state.player {
                Player::White => {
                    res.push_str(&format!(
                        "{}",
                        Square::from_file_rank(self.state.en_passant, 5)
                    ));
                }
                Player::Black => {
                    res.push_str(&format!(
                        "{}",
                        Square::from_file_rank(self.state.en_passant, 2)
                    ));
                }
            }
        } else {
            res.push('-');
        }
        res.push(' ');
        res.push_str(&format!("{}", self.state.move_clock));
        res.push(' ');
        res.push_str(&format!("{}", self.state.move_clock));
        res
    }
}
