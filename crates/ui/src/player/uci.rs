use crate::{board::RenderBoard, game::PlayedMove};

use super::Player;
use anyhow::{ensure, Context, Result};
use common::{
    board::{Board, UnmakeMove},
    MoveKind,
};
use move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{self, Child, ChildStdin, Stdio},
    sync::mpsc::{self, Receiver},
};
use uci::{req::GoRequest, resp::ResponseId};

pub struct UciPlayer {
    mov_gen: MoveGenerator,
    moves: InlineBuffer,
    recv: Receiver<uci::Response>,
    stdin: ChildStdin,
    _child: Child,
}

impl UciPlayer {
    pub fn new(path: &Path) -> Result<Self> {
        ensure!(path.exists(), "Engine file does not exists");
        let mut process = process::Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to spawn engine process")?;

        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();

        let (send, recv) = mpsc::channel();

        std::thread::spawn(move || {
            let _ = (|| {
                let mut buffer = String::new();
                let mut reader = BufReader::new(stdout);
                loop {
                    buffer.clear();
                    if reader.read_line(&mut buffer)? == 0 {
                        break;
                    }
                    if buffer.trim().is_empty() {
                        continue;
                    }
                    match buffer.trim().parse::<uci::Response>() {
                        Ok(x) => {
                            if send.send(x).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            println!("engine returned invalid uci response: {e}")
                        }
                    }
                }
                Result::Ok(())
            })()
            .map_err(|e: anyhow::Error| println!("engine read thread returned error: {e}"));
        });

        let mut this = UciPlayer {
            moves: InlineBuffer::new(),
            mov_gen: MoveGenerator::new(),
            _child: process,
            recv,
            stdin,
        };
        this.init()?;
        Ok(this)
    }

    fn init(&mut self) -> Result<()> {
        writeln!(self.stdin, "{}", uci::Request::Uci)?;

        loop {
            match self.recv.recv().unwrap() {
                uci::Response::UciOk => break,
                uci::Response::Id(ResponseId::Author(x)) => println!("engine authors: {x}"),
                uci::Response::Id(ResponseId::Name(x)) => println!("engine name: {x}"),
                _ => {}
            }
        }
        Ok(())
    }

    fn set_position(&mut self, board: &Board, board_moves: &[UnmakeMove]) {
        let moves = board_moves.iter().map(|x| x.mov.into()).collect();
        let mut board = board.clone();
        for m in board_moves.iter().rev() {
            board.unmake_move(*m);
        }

        writeln!(
            self.stdin,
            "{}",
            uci::Request::Position {
                fen: Some(Box::new(board)),
                moves
            }
        )
        .unwrap();
    }

    fn sync(&mut self) {
        writeln!(self.stdin, "{}", uci::Request::IsReady).unwrap();

        loop {
            if let uci::Response::ReadyOk = self.recv.recv().unwrap() {
                break;
            }
        }
    }

    fn go(&mut self, board: &RenderBoard) {
        let movetime = if board.white_time.is_some() {
            None
        } else {
            Some(1000)
        };

        writeln!(
            self.stdin,
            "{}",
            uci::Request::Go(GoRequest {
                winc: Some(board.increment.as_millis() as i32),
                binc: Some(board.increment.as_millis() as i32),
                wtime: board.white_time.map(|x| x.as_millis() as i64),
                btime: board.black_time.map(|x| x.as_millis() as i64),
                movetime,
                ..Default::default()
            })
        )
        .unwrap()
    }

    fn stop(&mut self) {
        writeln!(self.stdin, "{}", uci::Request::Stop).unwrap()
    }
}

impl Player for UciPlayer {
    fn update(&mut self, board: &mut crate::board::RenderBoard) -> PlayedMove {
        while let Ok(x) = self.recv.try_recv() {
            if let uci::Response::BestMove { r#move, .. } = x {
                println!("RECIEVED MOVE: {move}");
                let Some(m) = r#move.to_move(self.moves.as_slice()) else {
                    println!("MOVE NOT CORRECT");
                    return PlayedMove::Didnt;
                };

                board.highlight(m.from(), m.to());
                board.make_move(m);
                if m.kind() == MoveKind::Castle {
                    return PlayedMove::Castle;
                } else {
                    return PlayedMove::Move;
                }
            }
        }
        PlayedMove::Didnt
    }

    fn start_turn(&mut self, board: &crate::board::RenderBoard) {
        println!("STARTING TURN: {:?}", board.board.state.player);
        self.moves.clear();
        self.mov_gen
            .gen_moves::<gen_type::All>(&board.board, &mut self.moves);

        if self.moves.is_empty() {
            println!("NO MOVES");
            return;
        }

        self.set_position(board.board(), board.active_moves());
        self.sync();
        self.go(board);
    }

    fn stop(&mut self) {
        (*self).stop()
    }
}
