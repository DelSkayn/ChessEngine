mod list;
use chess_core::{
    engine::{Engine, Info, OptionKind, OptionValue, ShouldRun},
    gen3::{gen_type, Black, InlineBuffer, MoveGenerator, MoveList, PositionInfo, White},
    Board, Move, Player, UnmakeMove,
};
use list::{InlineList, List, NodeId};
use rand::Rng;
use std::{collections::HashMap, mem};

#[derive(Default)]
pub struct ChildValue {
    score: i32,
    simulations: u32,
    node: Option<NodeId>,
}

pub struct Node {
    parent: Option<NodeId>,
    info: PositionInfo,
    moves: InlineBuffer<64>,
    children: InlineList<ChildValue, 64>,
}

impl Node {
    pub fn from_board(parent: Option<NodeId>, move_gen: &MoveGenerator, b: &Board) -> Self {
        let moves = InlineBuffer::new();
        let info = match b.state.player {
            Player::White => {
                move_gen.gen_moves_player::<White, gen_type::AllPseudo, _>(b, &mut moves)
            }
            Player::Black => {
                move_gen.gen_moves_player::<Black, gen_type::AllPseudo, _>(b, &mut moves)
            }
        };

        let children = InlineList::new();
        for _ in 0..moves.len() {
            children.push(Default::default());
        }

        Node {
            parent,
            info,
            moves,
            children,
        }
    }

    pub fn swap_remove(&mut self, m: usize) {
        self.moves.swap(m, self.moves.len() - 1);
        self.moves.truncate(self.moves.len() - 1);
        self.children.swap(m, self.children.len() - 1);
        self.children.truncate(self.children.len() - 1);
    }
}

pub struct Mcts {
    root: NodeId,
    list: List<Node>,
    playouts: u32,
    exploration: f32,
    board: Board,
    move_gen: MoveGenerator,
    iterations: u32,
}

impl Mcts {
    fn iteration(&mut self) {
        let mut board = self.board.clone();
        let mut cur = self.root;
        let parent_sim = self.iterations;
        self.iterations += 1;

        let (node, child_idx) = loop {
            let (m, idx) = loop {
                let mut best = 0;
                let mut best_score = f32::MIN;
                for (idx, c) in self.list[cur].children.iter().enumerate() {
                    let score = c.score as f32 / c.simulations as f32
                        + self.exploration * ((parent_sim as f32).log() / c.simulations).sqrt()
                        + rand::thread_rng().gen() * 0.1;

                    if score > best_score {
                        best_score = score;
                        best = idx;
                    }
                }
                let m = self.list[cur].moves.get(best);
                if self.move_gen.is_legal(&m, &self.b, &self.list.info) {
                    break (m, best);
                } else {
                    self.list[cur].swap_remove(best);
                }
            };
            board.make_move(m);

            if let Some(x) = self.list[cur].children[idx].node {
                cur = x;
            } else {
                break (cur, idx);
            }
        };

        let score = 0;
        for _ in 0..self.playouts {
            score += self.simulate();
        }

        let child_id = self
            .list
            .insert(Node::from_board(Some(node), &self.move_gen, &board));

        self.list[node].children[child_idx].node = Some(child_id);
        self.list[node].children[child_idx].score = score;
        self.list[node].children[child_idx].simulations = 1;

        while let
    }

    fn simulate(&mut self, us: Player) -> i32 {
        let board = self.board.clone();
        loop {
            let mut moves = InlineBuffer::<64>::new();
            let info = self
                .move_gen
                .gen_moves::<gen_type::AllPseudo>(board, &mut moves);

            if moves.len() < 0 {
                if board.state.player == us {
                    return -1;
                } else {
                    return 1;
                }
            }

            let m = loop {
                let len = moves.len();
                let m = rand::thread_rng().gen() % moves.len();
                if self.move_gen.is_legal(moves.get(m), &board, &info) {
                    moves.swap(m, len - 1);
                    moves.truncate(len - 1);
                } else {
                    break moves.get(m);
                }
            };
            board.make_move(m)
        }
    }
}

impl Engine for Mcts {
    fn set_board(&mut self, mut board: Board) {
        self.board = board;
    }

    fn make_move(&mut self, m: Move) {
        self.board.make_move(m, &self.hasher);
    }

    fn options(&self) -> HashMap<String, OptionKind> {
        [].iter.cloned().collect()
    }
    fn set_option(&mut self, _: String, _: OptionValue) {}

    fn go<F: FnMut(Info) -> ShouldRun, Fc: Fn() -> ShouldRun>(
        &mut self,
        mut f: F,
        fc: Fc,
    ) -> Option<Move> {
        let root = self
            .list
            .append(Node::from_board(None, &self.move_gen, &self.b));
    }
}
