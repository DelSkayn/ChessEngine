use std::time::Duration;

use chess_core::{
    board::{EndChain, HashChain},
    gen::MoveGenerator,
    uci::{
        engine::{Engine, EngineSignal, SearchConfig, SearchResult},
        OptionMsg, OptionMsgType, Version,
    },
    Board as BaseBoard, Player,
};
pub use search::PvBuffer;

mod eval;
mod search;

pub type Board = BaseBoard<HashChain<EndChain>>;

pub struct Config {
    contempt: i32,
    hash_size: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            contempt: 100,
            hash_size: 16,
        }
    }
}

impl Config {
    pub fn options() -> Vec<OptionMsg> {
        vec![
            OptionMsg {
                name: "contempt".to_owned(),
                r#type: OptionMsgType::Spin {
                    default: 16,
                    max: Some(900),
                    min: Some(-100),
                },
            },
            OptionMsg {
                name: "Hash".to_owned(),
                r#type: OptionMsgType::Spin {
                    default: 16,
                    max: Some(1024 * 4),
                    min: Some(1),
                },
            },
        ]
    }

    pub fn set_option(&mut self, name: String, value: Option<String>) {
        match name.as_str() {
            "contempt" => {
                let Some(v) = value.and_then(|x| x.parse().ok()) else { return };
                if v < -100 || v > 900 {
                    return;
                }
                self.contempt = v;
            }
            "Hash" => {
                let Some(v) = value.and_then(|x| x.parse().ok()) else { return };
                if v < 1 || v > 1024 * 4 {
                    return;
                }
                self.hash_size = v;
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct SearchLimit {
    nodes: u64,
    time: Duration,
    depth: u8,
}

#[derive(Default)]
pub struct SearchInfo {
    nodes: u64,
}

pub struct AlphaBeta {
    config: Config,
    limit: SearchLimit,
    info: SearchInfo,
    board: Board,
    pv: PvBuffer<{ Self::MAX_DEPTH as usize }>,
    gen: MoveGenerator,
}

impl AlphaBeta {
    pub fn new() -> Self {
        AlphaBeta {
            config: Config::default(),
            limit: SearchLimit::default(),
            info: SearchInfo::default(),
            board: Board::start_position(HashChain::new()),
            pv: PvBuffer::new(),
            gen: MoveGenerator::new(),
        }
    }
}

impl Engine for AlphaBeta {
    const NAME: &'static str = "Alpha Beta";
    const AUTHOR: &'static str = "Mees Delzenne";

    fn version() -> Option<Version> {
        let major = env!("CARGO_PKG_VERSION_MAJOR").parse().ok()?;
        let minor = env!("CARGO_PKG_VERSION_MINOR").parse().ok()?;
        let patch = env!("CARGO_PKG_VERSION_PATCH").parse().ok()?;
        Some(Version {
            major,
            minor,
            patch,
        })
    }

    fn options() -> Vec<OptionMsg> {
        Config::options()
    }

    fn set_option(&mut self, name: String, value: Option<String>) {
        self.config.set_option(name, value);
    }

    fn new_position(&mut self, board: chess_core::Board) {
        self.board.copy_position(&board);
    }

    fn make_move(&mut self, r#move: chess_core::Move) {
        self.board.make_move(r#move);
    }

    fn search(&mut self, config: SearchConfig, signal: EngineSignal) -> SearchResult {
        let player = self.board.state.player;

        let limit = if config.infinite {
            Duration::MAX
        } else if let Some(x) = config.movetime {
            x
        } else {
            if let (Player::White, Some(time), _) | (Player::Black, _, Some(time)) =
                (player, config.wtime, config.btime)
            {
                let increment = config.winc.unwrap_or_default();
                let time_left = time + increment * 20;
                time_left / 20
            } else {
                Duration::MAX
            }
        };
        self.limit.time = limit;
        self.limit.nodes = config.nodes.unwrap_or(u64::MAX);
        self.limit.depth = config.depth.map(|x| x.min(255) as u8).unwrap_or(255);

        self.info = SearchInfo::default();

        let r#move = self.search_root(signal);
        SearchResult {
            r#move,
            ponder: None,
        }
    }
}
