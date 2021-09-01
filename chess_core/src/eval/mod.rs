use crate::{
    engine::{Engine, Info, OptionKind, OptionValue, ShouldRun},
    gen3::{gen_type, InlineBuffer, MoveGenerator, MoveList},
    hash::Hasher,
    util::{BoardArray, PieceArray},
    Board, Move, Piece, Player,
};
use std::{collections::HashMap as RHashMap, mem};

mod search;

#[derive(Clone, Copy)]
pub enum StoredValue {
    Exact(i32),
    UpperBound(i32),
    LowerBound(i32),
}

impl Default for StoredValue {
    fn default() -> Self {
        StoredValue::Exact(0)
    }
}

#[derive(Default, Clone, Copy)]
pub struct Stored {
    hash: u64,
    best_move: u16,
    depth: u16,
    value: StoredValue,
}

pub struct HashMap {
    bitmap: u64,
    values: Box<[Stored]>,
}

impl HashMap {
    pub fn new(size: usize) -> Self {
        let actual = size.next_power_of_two() >> 1;
        let bitmap = (actual - 1) as u64;
        let values = vec![Stored::default(); actual];
        HashMap {
            bitmap,
            values: values.into_boxed_slice(),
        }
    }

    pub fn new_from_mb(mb: usize) -> Self {
        let bytes = mb << 20;
        let values = bytes / mem::size_of::<Stored>();
        let values = values.next_power_of_two() >> 1;
        let bitmap = (values - 1) as u64;
        let values = vec![Stored::default(); values];
        HashMap {
            bitmap,
            values: values.into_boxed_slice(),
        }
    }

    #[inline]
    pub fn lookup(&self, hash: u64) -> Option<&Stored> {
        let idx = self.bitmap & hash;
        let res = unsafe { self.values.get_unchecked(idx as usize) };
        if res.hash == hash {
            Some(res)
        } else {
            None
        }
    }

    #[inline]
    pub fn store(&mut self, v: Stored) {
        let idx = v.hash & self.bitmap;
        unsafe {
            *self.values.get_unchecked_mut(idx as usize) = v;
        }
    }
}

pub struct Eval {
    gen: MoveGenerator,
    hasher: Hasher,
    hashmap: HashMap,
    nodes_evaluated: usize,
    table_hits: usize,
    cut_offs: usize,
    value_lookup: PieceArray<i32>,
    board: Board,
}

impl Engine for Eval {
    fn set_board(&mut self, mut board: Board) {
        board.calc_hash(&self.hasher);
        self.board = board;
    }

    fn make_move(&mut self, m: Move) {
        self.board.make_move(m);
    }

    fn options(&self) -> RHashMap<String, OptionKind> {
        [(
            "Hash".to_string(),
            OptionKind::Spin {
                default: 16,
                min: Some(1),
                max: Some(256),
            },
        )]
        .iter()
        .cloned()
        .collect()
    }

    fn set_option(&mut self, name: String, value: OptionValue) {
        match (name.as_str(), value) {
            ("Hash", OptionValue::Spin(x)) => self.hashmap = HashMap::new_from_mb(x as usize),
            _ => {}
        }
    }

    fn go<F: FnMut(Info) -> ShouldRun, Fc: Fn() -> ShouldRun>(
        &mut self,
        mut f: F,
        fc: Fc,
    ) -> Option<Move> {
        let redo = [200, i32::MAX];

        let mut moves = InlineBuffer::<128>::new();
        let mut b = self.board.clone();
        self.gen.gen_moves::<gen_type::All, _>(&mut b, &mut moves);
        self.nodes_evaluated = 0;
        self.table_hits = 0;
        self.cut_offs = 0;
        let mut depth = 0;
        let mut best_move = None;
        let best_move_idx = self.hashmap.lookup(b.hash).map(|x| x.best_move);
        self.order_moves(&b, &mut moves, best_move_idx);

        let mut best_move_idx = best_move_idx.unwrap_or(0);

        let mut alpha = -Self::CHECK_VALUE;
        let mut beta = Self::CHECK_VALUE;

        let mut redo_alpha = 0;
        let mut redo_beta = 0;
        'main: loop {
            f(Info::Depth(depth));

            let mut value = match b.state.player {
                Player::White => -i32::MAX,
                Player::Black => i32::MAX,
            };
            for (idx, m) in moves.iter().copied().enumerate() {
                let prev = b.clone();
                let undo = b.make_move(m);
                //assert!(b.is_valid());
                let move_value = match b.state.player {
                    Player::Black => self.alpha_beta_min(&mut b, value, beta, depth, &fc),
                    Player::White => self.alpha_beta_max(&mut b, alpha, value, depth, &fc),
                };
                b.unmake_move(undo);
                assert_eq!(prev, b);

                match b.state.player {
                    Player::White => {
                        if move_value > value {
                            best_move = Some(m);
                            best_move_idx = idx as u16;
                            value = move_value;
                        }
                    }
                    Player::Black => {
                        if move_value < value {
                            best_move = Some(m);
                            best_move_idx = idx as u16;
                            value = move_value;
                        }
                    }
                }

                if fc() == ShouldRun::Stop {
                    break 'main;
                }
            }

            if value <= alpha {
                alpha = alpha.saturating_sub(redo[redo_alpha]);
                redo_alpha += 1;
                continue;
            }
            if value >= beta {
                beta = beta.saturating_add(redo[redo_beta]);
                redo_beta += 1;
                continue;
            }

            redo_alpha = 0;
            redo_beta = 0;

            let cont = if let Some(mov) = best_move {
                f(Info::BestMove { mov, value })
            } else {
                ShouldRun::Continue
            }
            .chain(f(Info::Nodes(self.nodes_evaluated)))
            .chain(f(Info::TransHit(self.table_hits)))
            .chain(f(Info::Round));

            if cont == ShouldRun::Stop {
                break;
            }

            alpha = value.saturating_sub(Self::PAWN_VALUE / 2 + 1);
            beta = value.saturating_add(Self::PAWN_VALUE / 2 + 1);

            moves.swap(best_move_idx as usize, 0);

            depth += 1;
        }
        best_move
    }
}

#[derive(Default)]
pub struct BestMove {
    pub mov: Option<Move>,
    pub value: i32,
    pub depth: usize,
    pub nodes_evaluated: usize,
    pub table_hits: usize,
    pub cut_offs: usize,
}

#[derive(Default, Debug)]
pub struct Buffers {
    root: Vec<Move>,
    depth: Vec<Vec<Move>>,
}

impl Eval {
    pub const PAWN_VALUE: i32 = 100;
    pub const KNIGHT_VALUE: i32 = 320;
    pub const BISHOP_VALUE: i32 = 325;
    pub const ROOK_VALUE: i32 = 500;
    pub const QUEEN_VALUE: i32 = 975;

    pub const CHECK_VALUE: i32 = i32::MAX - 1;

    const PAWN_TABLE: BoardArray<i32> = BoardArray::new_array([
        0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5,
        5, 10, 27, 27, 10, 5, 5, 0, 0, 0, 25, 25, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10,
        -25, -25, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);

    const KNIGHT_TABLE: BoardArray<i32> = BoardArray::new_array([
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15,
        10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15,
        15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -20, -30, -30, -20, -40, -50,
    ]);

    const BISHOP_TABLE: BoardArray<i32> = BoardArray::new_array([
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5,
        0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -40, -10, -10, -40, -10, -20,
    ]);

    const KING_TABLE: BoardArray<i32> = BoardArray::new_array([
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
        -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20,
        30, 10, 0, 0, 10, 30, 20,
    ]);

    /*
    const KING_END_TABLE: BoardArray<i32> = BoardArray::new_array([
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20,
    30, 30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10,
    -30, -30, -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30,
    -30, -30, -30, -30, -50,
    ]);
    */

    pub fn new() -> Self {
        let hasher = Hasher::new();
        let mut value_lookup = PieceArray::new(0);
        value_lookup[Piece::WhiteKing] = 9999999;
        value_lookup[Piece::WhiteQueen] = Self::QUEEN_VALUE;
        value_lookup[Piece::WhiteRook] = Self::ROOK_VALUE;
        value_lookup[Piece::WhiteBishop] = Self::BISHOP_VALUE;
        value_lookup[Piece::WhiteKnight] = Self::KNIGHT_VALUE;
        value_lookup[Piece::WhitePawn] = Self::PAWN_VALUE;
        value_lookup[Piece::BlackKing] = 9999999;
        value_lookup[Piece::BlackQueen] = Self::QUEEN_VALUE;
        value_lookup[Piece::BlackRook] = Self::ROOK_VALUE;
        value_lookup[Piece::BlackBishop] = Self::BISHOP_VALUE;
        value_lookup[Piece::BlackKnight] = Self::KNIGHT_VALUE;
        value_lookup[Piece::BlackPawn] = Self::PAWN_VALUE;
        let mut board = Board::start_position();
        board.calc_hash(&hasher);

        Eval {
            hasher,
            hashmap: HashMap::new_from_mb(16),
            gen: MoveGenerator::new(),
            nodes_evaluated: 0,
            table_hits: 0,
            cut_offs: 0,
            value_lookup,
            board,
        }
    }
}
