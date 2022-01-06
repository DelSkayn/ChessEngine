#![allow(unused_imports)]
#![allow(dead_code)]

mod list;
use chess_core::{
    board::{Board, EndChain},
    engine::{Engine, EngineControl, Info, OptionKind, OptionValue},
    gen::{gen_type, Black, InlineBuffer, MoveGenerator, MoveList, PositionInfo, White},
    hash::Hasher,
    Move, Piece, Player, UnmakeMove,
};
use list::{InlineVec, List, NodeId};
use rand::Rng;
use std::{collections::HashMap, fs, io, mem};

pub struct Node {
    parent: Option<NodeId>,
    moves: InlineBuffer<128>,
    info: PositionInfo,
    simulations: u32,
    score: f32,
    children: InlineVec<(NodeId, Move), 128>,
}

impl Node {
    pub fn new(parent: Option<NodeId>, b: &Board, move_gen: &MoveGenerator) -> Self {
        let mut moves = InlineBuffer::new();
        let info = move_gen.gen_moves::<gen_type::All, _, _>(&b, &mut moves);
        Node {
            parent,
            info,
            simulations: 0,
            score: 0.0,
            moves,
            children: InlineVec::new(),
        }
    }

    pub fn fully_expanded(&self) -> bool {
        self.moves.len() == 0
    }
}

pub struct Options {
    max_rollout: usize,
    exploration: f32,
    playouts: u32,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            max_rollout: 10_000,
            exploration: (2.0f32).sqrt(),
            playouts: 3,
        }
    }
}

pub struct Mcts<C> {
    root: NodeId,
    list: List<Node>,
    pub options: Options,
    board: Board,
    move_gen: MoveGenerator,
    iterations: u32,
    pub retry_quites: bool,
    control: C,
}

impl<C: EngineControl> Mcts<C> {
    const SCORE_WIN: f32 = 1.0;
    const SCORE_DRAW: f32 = 0.5;
    const SCORE_LOSE: f32 = 0.0;

    pub fn new() -> Self {
        let mut list = List::new();
        let board = Board::start_position(EndChain);
        let move_gen = MoveGenerator::new();
        Mcts {
            options: Default::default(),
            root: list.insert(Node::new(None, &board, &move_gen)),
            list,
            board,
            move_gen,
            iterations: 0,
            retry_quites: false,
            control: C::default(),
        }
    }

    fn iteration(&mut self) {
        let mut board = self.board.clone();
        let mut cur_node = self.root;
        let mut rng = rand::thread_rng();

        // Selection
        loop {
            if self.list[cur_node].fully_expanded() {
                let mut best = None;
                let mut best_score = f32::MIN;

                for (c, m) in self.list[cur_node].children.iter().copied() {
                    let n = &self.list[c];
                    let score = n.score + rng.gen::<f32>() * 0.1;
                    let simulations = n.simulations as f32;
                    let root_simulations = self.list[cur_node].simulations as f32;

                    let score = score / simulations
                        + self.options.exploration * (root_simulations.ln() / simulations);
                    if score > best_score {
                        best_score = score;
                        best = Some((c, m));
                    }
                }

                if let Some((node_id, mov)) = best {
                    board.make_move(mov);
                    cur_node = node_id;
                } else {
                    break;
                }
            } else {
                let pick = rng.gen::<usize>() % self.list[cur_node].moves.len();
                let mov = self.list[cur_node].moves.get(pick);
                self.list[cur_node].moves.swap_remove(pick);
                board.make_move(mov);
                let old_node = cur_node;
                cur_node = self
                    .list
                    .insert(Node::new(Some(old_node), &board, &self.move_gen));
                self.list[old_node].children.push((cur_node, mov));
                break;
            }
        }

        // Simulate
        let mut score = self.simulate(cur_node, &board, &mut rng);

        // Propagate
        loop {
            self.list[cur_node].simulations += self.options.playouts;
            self.list[cur_node].score += score;
            score = self.options.playouts as f32 * Self::SCORE_WIN - score;
            if let Some(p) = self.list[cur_node].parent {
                cur_node = p;
            } else {
                break;
            }
        }
    }

    fn simulate(&mut self, node: NodeId, board: &Board, rng: &mut impl rand::Rng) -> f32 {
        const MAX_ROLLOUT: usize = 10_000;

        let node = &self.list[node];
        let mut score = 0.0;

        // No moves for node, it is either a checkmate or a stalemate
        if node.moves.len() == 0 {
            if (node.info.attacked & board.pieces[Piece::player_king(board.state.player)]).any() {
                return Self::SCORE_WIN * self.options.playouts as f32;
            } else {
                return self.options.playouts as f32 * Self::SCORE_DRAW;
            }
        }

        for _ in 0..self.options.playouts {
            let mut b = board.clone();
            let pick = rng.gen::<usize>() % node.moves.len();
            let first_move = node.moves.get(pick);
            b.make_move(first_move);
            let mut move_buffer = InlineBuffer::<128>::new();
            let mut info = self
                .move_gen
                .gen_moves::<gen_type::AllPseudo, _, _>(&b, &mut move_buffer);

            'rollout: for i in 0..MAX_ROLLOUT {
                if self.move_gen.drawn(&b, &info) {
                    score += Self::SCORE_DRAW;
                    break;
                }

                let mov = loop {
                    if move_buffer.len() == 0 {
                        if (node.info.attacked & b.pieces[Piece::player_king(b.state.player)]).any()
                        {
                            if b.state.player == board.state.player {
                                score += Self::SCORE_WIN;
                            }
                        } else {
                            score += Self::SCORE_DRAW;
                        }
                        break 'rollout;
                    }

                    let pick = rng.gen::<usize>() % move_buffer.len();
                    let mov = move_buffer.get(pick);
                    if self.move_gen.is_legal(mov, &b, &info) {
                        if move_buffer.len() > 1 && self.should_retry(mov, &b, rng) {
                            move_buffer.swap_remove(pick);
                        } else {
                            break mov;
                        }
                    } else {
                        move_buffer.swap_remove(pick);
                    }
                };

                b.make_move(mov);
                move_buffer.clear();
                info = self
                    .move_gen
                    .gen_moves::<gen_type::AllPseudo, _, _>(&b, &mut move_buffer);

                if i == MAX_ROLLOUT - 1 {
                    score += Self::SCORE_DRAW;
                    break;
                }
            }
        }
        score
    }

    fn should_retry(&self, mov: Move, b: &Board, rng: &mut impl rand::Rng) -> bool {
        if !self.retry_quites {
            return false;
        }

        rng.gen::<f32>() < 0.5
            && (b.on(mov.to()).is_none()
                || mov.ty() == Move::TYPE_PROMOTION
                    && mov.promotion_piece() != Move::PROMOTION_QUEEN)
    }

    pub fn dump_tree(&self) {
        use io::Write;
        let mut file = fs::File::create("mcts.dot").unwrap();
        writeln!(file, "digraph mcts{{").unwrap();
        writeln!(file, "{} [label=\"root\"];", self.root.0).unwrap();
        self.dump_tree_rec(&mut file, self.root).unwrap();
        writeln!(file, "}}").unwrap();
    }

    fn dump_tree_rec(&self, f: &mut impl io::Write, node: NodeId) -> io::Result<()> {
        let mut best_score = f32::MIN;
        let mut best = None;
        for (c, m) in self.list[node].children.iter().copied() {
            writeln!(
                f,
                "{} [shape=record, label=\"{{ {}|{} }}\"];",
                c.0,
                m,
                self.list[c].score / self.list[c].simulations as f32
            )?;
            writeln!(f, "{} -> {};", node.0, c.0)?;
            let n = &self.list[c];
            if n.score > best_score {
                best_score = n.score;
                best = Some(c);
            }
        }
        if let Some(best) = best {
            self.dump_tree_rec(f, best)?;
        }
        Ok(())
    }
}

impl<C: EngineControl> Engine<C> for Mcts<C> {
    const NAME: &'static str = "Random play MCTS";

    fn set_board(&mut self, board: Board) {
        self.board = board;
    }

    fn make_move(&mut self, m: Move) {
        self.board.make_move(m);
    }

    fn options(&self) -> HashMap<String, OptionKind> {
        [
            (
                "playouts".to_string(),
                OptionKind::Spin {
                    default: 3,
                    max: Some(20),
                    min: Some(1),
                },
            ),
            ("exploration".to_string(), OptionKind::String),
            (
                "max_rollout".to_string(),
                OptionKind::Spin {
                    default: 10_000,
                    max: Some(i32::MAX),
                    min: Some(20),
                },
            ),
        ]
        .iter()
        .cloned()
        .collect()
    }
    fn set_option(&mut self, name: String, value: OptionValue) {
        match name.as_str() {
            "playouts" => {
                if let OptionValue::Spin(x) = value {
                    self.options.playouts = x as u32;
                }
            }
            "exploration" => {
                if let OptionValue::String(x) = value {
                    if let Ok(x) = x.parse() {
                        self.options.exploration = x;
                    }
                }
            }
            "max_rollout" => {
                if let OptionValue::Spin(x) = value {
                    self.options.max_rollout = x as usize;
                }
            }
            _ => {}
        }
    }

    fn go(
        &mut self,
        control: C,
        _time_left: Option<std::time::Duration>,
        _limit: chess_core::engine::EngineLimit,
    ) -> Option<Move> {
        self.control = control;

        self.iterations = 0;
        self.list.clear();
        self.root = self
            .list
            .insert(Node::new(None, &self.board, &self.move_gen));

        if self.list[self.root].moves.len() == 0 {
            return None;
        }

        while !self.control.should_stop() {
            self.iteration();
            self.iterations += 1;
        }

        self.control
            .info(Info::Debug(format!("Iterations: {}", self.iterations)));

        let mut most_simulations = 0;
        let mut m = None;
        let mut score = 0.0;
        for (c, mov) in self.list[self.root].children.iter().copied() {
            let sim = self.list[c].simulations;
            let cur_score = self.list[c].score;
            self.control.info(Info::Debug(format!(
                "{}:{} = {}",
                mov,
                sim,
                cur_score / sim as f32
            )));
            if sim > most_simulations {
                most_simulations = sim;
                score = cur_score / sim as f32;
                m = Some(mov);
            }
        }

        self.control.info(Info::Debug(format!("score: {}", score)));

        self.dump_tree();

        return m;
    }
}
