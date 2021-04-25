use crate::{
    gen::{InlineBuffer, MoveBuffer, MoveGenerator},
    hash::Hasher,
    util::{BoardArray, PieceArray},
    Board, Move, Piece,
};
use std::sync::{Arc, atomic::{Ordering,AtomicBool}};

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
    stop: Arc<AtomicBool>,
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

    pub fn new(hasher: Hasher, hash_size: usize,stop: Arc<AtomicBool>) -> Self {
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
        Eval {
            hasher,
            hashmap: HashMap::new(hash_size),
            gen: MoveGenerator::new(),
            nodes_evaluated: 0,
            table_hits: 0,
            cut_offs: 0,
            value_lookup,
            stop,
        }
    }

    pub fn eval(&mut self, b: &Board, cont: &mut impl FnMut(Option<BestMove>) -> bool) {
        let redo = [200, i32::MAX];

        let mut moves = InlineBuffer::new();
        self.gen.gen_moves(&b, &mut moves);
        self.nodes_evaluated = 0;
        self.table_hits = 0;
        self.cut_offs = 0;
        let mut depth = 0;
        let mut b = b.clone();
        let mut best_move = None;
        let best_move_idx = self.hashmap.lookup(b.hash).map(|x| x.best_move);
        self.order_moves(&b,&mut moves, best_move_idx);
        let white_turn = b.white_turn();

        let mut best_move_idx = best_move_idx.unwrap_or(0);

        let mut alpha = -Self::CHECK_VALUE;
        let mut beta = Self::CHECK_VALUE;

        let mut redo_alpha = 0;
        let mut redo_beta = 0;
        loop {
            let mut value = if white_turn { -i32::MAX } else { i32::MAX };
            for (idx, m) in moves.iter().copied().enumerate() {
                let capture = b.on(m.to()).is_some();
                let undo = b.make_move(m, &self.hasher);
                let move_value = if white_turn {
                    self.alpha_beta_min(&mut b, value, beta, depth,capture)
                } else {
                    self.alpha_beta_max(&mut b, alpha, value, depth,capture)
                };
                b.unmake_move(undo, &self.hasher);

                if white_turn {
                    if move_value > value {
                        best_move = Some(m);
                        best_move_idx = idx as u16;
                        value = move_value;
                    }
                } else {
                    if move_value < value {
                        best_move = Some(m);
                        best_move_idx = idx as u16;
                        value = move_value;
                    }
                }

                if !cont(None) {
                    return;
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

            let best_move = BestMove {
                mov: best_move,
                value,
                depth: depth as usize,
                nodes_evaluated: self.nodes_evaluated,
                table_hits: self.table_hits,
                cut_offs: self.cut_offs,
            };

            if !cont(Some(best_move)) {
                return;
            }

            alpha = value - Self::PAWN_VALUE / 2;
            beta = value + Self::PAWN_VALUE / 2;

            moves.swap(best_move_idx as usize,0);

            depth += 1;
        }
    }

    fn alpha_beta_max(
        &mut self,
        b: &mut Board,
        mut alpha: i32,
        mut beta: i32,
        depth: u16,
        was_capture: bool
    ) -> i32 {
        if self.stop.load(Ordering::Acquire){
            let color = if b.white_turn() { -1 } else { 1 };
            return color * Self::CHECK_VALUE;
        }

        let mut stored_best_move = None;
        if let Some(x) = self.hashmap.lookup(b.hash) {
            stored_best_move = Some(x.best_move);
            self.table_hits += 1;
            if x.depth >= depth {
                match x.value {
                    StoredValue::Exact(x) => return x,
                    StoredValue::LowerBound(x) => {
                        alpha = alpha.max(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                    StoredValue::UpperBound(x) => {
                        beta = beta.min(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                }
            }
        }

        if depth == 0 {
            if was_capture{
                return self.quiesce_max(b, alpha, beta);
            }else{
                return self.eval_board(b);
            }
        }

        let mut buffer = InlineBuffer::new();
        self.gen.gen_moves(b, &mut buffer);
        if buffer.len() == 0 {
            let color = if b.white_turn() { -1 } else { 1 };
            return color * Self::CHECK_VALUE;
        }

        self.order_moves(b,&mut buffer, stored_best_move);

        let mut best_move = 0u16;

        for (idx, m) in buffer.iter().copied().enumerate() {
            #[cfg(debug_assertions)]
            let tmp = b.clone();
            let capture = b.on(m.to()).is_some();
            let undo = b.make_move(m, &self.hasher);
            debug_assert!(b.is_valid());
            let value = self.alpha_beta_min(b, alpha, beta, depth - 1,capture);
            b.unmake_move(undo, &self.hasher);
            #[cfg(debug_assertions)]
            debug_assert!(tmp.is_equal(b),"{:#?}",b);
            if value > alpha {
                best_move = idx as u16;
                alpha = value;
            }
            if value >= beta {
                self.cut_offs += 1;
                break;
            }
        }

        if alpha >= beta {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth,
                value: StoredValue::LowerBound(alpha),
                best_move,
            })
        } else {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth,
                value: StoredValue::Exact(alpha),
                best_move,
            })
        }

        return alpha;
    }

    fn alpha_beta_min(
        &mut self,
        b: &mut Board,
        mut alpha: i32,
        mut beta: i32,
        depth: u16,
        was_capture: bool,
    ) -> i32 {
        if self.stop.load(Ordering::Acquire){
            return -Self::CHECK_VALUE;
        }

        let mut stored_best_move = None;
        if let Some(x) = self.hashmap.lookup(b.hash) {
            stored_best_move = Some(x.best_move);
            self.table_hits += 1;
            if x.depth >= depth {
                match x.value {
                    StoredValue::Exact(x) => return x,
                    StoredValue::LowerBound(x) => {
                        alpha = alpha.max(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                    StoredValue::UpperBound(x) => {
                        beta = beta.min(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                }
            }
        }

        if depth == 0 {
            if was_capture{
            return self.quiesce_min(b, alpha, beta);
            }else{
                return self.eval_board(b);
            }
        }

        let mut buffer = InlineBuffer::new();
        self.gen.gen_moves(b, &mut buffer);
        if buffer.len() == 0 {
            return -Self::CHECK_VALUE;
        }

        self.order_moves(b,&mut buffer, stored_best_move);

        let mut best_move = 0;

        for (idx, m) in buffer.iter().copied().enumerate() {
            #[cfg(debug_assertions)]
            let tmp = b.clone();
            let capture = b.on(m.to()).is_some();
            let undo = b.make_move(m, &self.hasher);
            debug_assert!(b.is_valid());
            let value = self.alpha_beta_max(b, alpha, beta, depth - 1,capture);
            b.unmake_move(undo, &self.hasher);
            #[cfg(debug_assertions)]
            debug_assert!(tmp.is_equal(b),"{:#?}",b);
            if value < beta {
                best_move = idx as u16;
                beta = value;
            }
            if beta <= alpha {
                self.cut_offs += 1;
                break;
            }
        }

        if alpha >= beta {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth,
                value: StoredValue::UpperBound(beta),
                best_move,
            })
        } else {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth,
                value: StoredValue::Exact(beta),
                best_move,
            })
        }

        return beta;
    }

    fn quiesce_max(&mut self, b: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        let value = self.eval_board(b);
        if value >= beta {
            return beta;
        }
        if alpha < value {
            alpha = value
        }

        let mut buffer = InlineBuffer::new();
        self.gen.gen_moves(b, &mut buffer);
        self.order_moves(b,&mut buffer, None);
        for m in buffer.iter().copied() {
            if b.on(m.to()).is_none(){
                continue;
            }
            #[cfg(debug_assertions)]
            let tmp = b.clone();
            let undo = b.make_move(m, &self.hasher);
            debug_assert!(b.is_valid(),"{:#?}",b);
            let value = self.quiesce_min(b, alpha, beta);
            b.unmake_move(undo, &self.hasher);
            #[cfg(debug_assertions)]
            debug_assert!(tmp.is_equal(b),"{:#?}",b);

            if value >= beta {
                return beta;
            }
            if value > alpha {
                alpha = value
            }
        }
        alpha
    }

    fn quiesce_min(&mut self, b: &mut Board, alpha: i32, mut beta: i32) -> i32 {
        let value = self.eval_board(b);
        if value <= alpha {
            return alpha;
        }
        if value < beta {
            beta = value
        }

        let mut buffer = InlineBuffer::new();
        self.gen.gen_moves(b, &mut buffer);
        self.order_moves(b,&mut buffer, None);
        for m in buffer.iter().copied() {
            if b.on(m.to()).is_none(){
                continue;
            }
            #[cfg(debug_assertions)]
            let tmp = b.clone();
            let undo = b.make_move(m, &self.hasher);
            assert!(b.is_valid(),"{:#?}",b);
            let value = self.quiesce_max(b, alpha, beta);
            b.unmake_move(undo, &self.hasher);
            #[cfg(debug_assertions)]
            assert!(tmp.is_equal(b),"{:#?}",b);
            if value <= alpha {
                return alpha;
            }
            if value < beta {
                beta = value
            }
        }
        beta
    }

    /*
    fn negamax(
        &mut self,
        b: &mut Board,
        buffers: &mut [Vec<Move>],
        mut alpha: i32,
        mut beta: i32,
        color: i32,
    ) -> i32 {
        let old_alpha = alpha;

        let mut stored_best_move = None;

        if let Some(x) = self.hashmap.lookup(b.hash) {
            stored_best_move = Some(x.best_move);
            self.table_hits += 1;
            if x.depth >= buffers.len() as u32 {
                match x.value {
                    StoredValue::Exact(x) => return x,
                    StoredValue::LowerBound(x) => {
                        alpha = alpha.max(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                    StoredValue::UpperBound(x) => {
                        beta = beta.min(x);
                        if alpha >= beta {
                            self.cut_offs += 1;
                            return x;
                        }
                    }
                }
            }
        }

        if buffers.is_empty() {
            return color * self.eval_board(b, color);
        }

        let (buffer, rest) = buffers.split_first_mut().unwrap();
        buffer.clear();
        self.gen.gen_moves(b, buffer);

        if buffer.is_empty() {
            return -i32::MAX;
        }

        self.order_moves(buffer, stored_best_move);

        let mut value = -i32::MAX;
        let mut best_move = 0;

        //dbg!(buffer.len());
        for (idx, m) in buffer.iter().copied().enumerate() {
            let tmp = b.clone();
            let undo = b.make_move(m, &self.hasher);
            b.assert_valid();
            let new_value = -self.negamax(b, rest, -beta, -alpha, -color);
            b.unmake_move(undo, &self.hasher);
            assert_eq!(tmp, *b);
            if value < new_value {
                best_move = idx as u32;
                value = new_value;
            }
            alpha = alpha.max(value);
            if alpha >= beta {
                self.cut_offs += 1;
                break;
            }
        }

        if value <= old_alpha {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth: buffers.len() as u32,
                value: StoredValue::UpperBound(value),
                best_move,
            })
        } else if value >= beta {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth: buffers.len() as u32,
                value: StoredValue::LowerBound(value),
                best_move,
            })
        } else {
            self.hashmap.store(Stored {
                hash: b.hash,
                depth: buffers.len() as u32,
                value: StoredValue::Exact(value),
                best_move,
            })
        }

        return value;
    }
    */

    pub fn eval_board(&mut self, b: &Board) -> i32 {
        self.nodes_evaluated += 1;

        if self.gen.check_mate(b) {
            let color = if b.white_turn() { -1 } else { 1 };
            return color * Self::CHECK_VALUE;
        }

        let mut piece_value = (b[Piece::WhiteQueen].count() as i32
            - b[Piece::BlackQueen].count() as i32)
            * Self::QUEEN_VALUE
            + (b[Piece::WhiteRook].count() as i32 - b[Piece::BlackRook].count() as i32)
                * Self::ROOK_VALUE
            + (b[Piece::WhiteBishop].count() as i32 - b[Piece::BlackBishop].count() as i32)
                * Self::BISHOP_VALUE
            + (b[Piece::WhiteKnight].count() as i32 - b[Piece::BlackKnight].count() as i32)
                * Self::KNIGHT_VALUE
            + (b[Piece::WhitePawn].count() as i32 - b[Piece::BlackPawn].count() as i32)
                * Self::PAWN_VALUE;

        for p in b[Piece::WhiteKing].iter() {
            piece_value += Self::KING_TABLE[p.flip()]
        }
        for p in b[Piece::WhiteBishop].iter() {
            piece_value += Self::BISHOP_TABLE[p.flip()]
        }
        for p in b[Piece::WhiteKnight].iter() {
            piece_value += Self::KNIGHT_TABLE[p.flip()]
        }
        for p in b[Piece::WhitePawn].iter() {
            piece_value += Self::PAWN_TABLE[p.flip()]
        }
        for p in b[Piece::BlackKing].iter() {
            piece_value -= Self::KING_TABLE[p]
        }
        for p in b[Piece::BlackBishop].iter() {
            piece_value -= Self::BISHOP_TABLE[p]
        }
        for p in b[Piece::BlackKnight].iter() {
            piece_value -= Self::KNIGHT_TABLE[p]
        }
        for p in b[Piece::BlackPawn].iter() {
            piece_value -= Self::PAWN_TABLE[p]
        }

        piece_value
    }

    pub fn order_moves<T>(&mut self,b: &Board, moves: &mut T, stored_best_move: Option<u16>)
    where
        T: MoveBuffer,
    {
        let move_swap = if let Some(x) = stored_best_move {
            if moves.len() > x as usize {
                moves.swap(0, x as usize);
                1
            } else {
                0
            }
        } else {
            0
        };

        if moves.len() == move_swap {
            return;
        }

        let mut best_value = self.eval_move(b,moves.get(move_swap));
        for i in move_swap + 1..moves.len() {
            let v = self.eval_move(b,moves.get(move_swap));
            if v > best_value {
                moves.swap(move_swap, i);
                best_value = v;
            }
        }
    }

    pub fn eval_move(&mut self, b: &Board,mov: &Move) -> i32 {
        let mut value = 0;
        if let Some(taken) = b.on(mov.to()){
            value += self.value_lookup[taken] * 100 + self.value_lookup[b.on(mov.from()).unwrap()]
        }
        match mov.ty() {
            Move::TYPE_CASTLE => value += 8,
            Move::TYPE_PROMOTION => value += Self::PAWN_VALUE,
            _ => {},
        }
        value
    }
}
