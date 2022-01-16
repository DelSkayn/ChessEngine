use anyhow::{anyhow, Result};
use chess_core::{board::EndChain, gen::MoveGenerator, Board};
use chess_uci::UciMove;
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{self, Child, ChildStdin, ChildStdout, Stdio},
    time::{Duration, Instant},
};

use crate::GameOutcome;

pub struct Engine {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    child: Child,
}

impl Engine {
    pub fn from_path(p: &Path) -> Result<Self> {
        let mut child = process::Command::new(p)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut p = Engine {
            stdin: child.stdin.take().unwrap(),
            stdout: BufReader::new(child.stdout.take().unwrap()),
            child,
        };

        writeln!(p.stdin, "uci")?;

        Ok(p)
    }

    pub fn run(
        &mut self,
        start_fen: &str,
        moves: &[UciMove],
        b: &Board,
        wtime: Duration,
        btime: Duration,
    ) -> Result<UciMove> {
        write!(self.stdin, "position fen {}", start_fen)?;
        if !moves.is_empty() {
            write!(self.stdin, "moves")?;
            for m in moves.iter() {
                write!(self.stdin, " {}", m)?;
            }
        }
        writeln!(self.stdin)?;
        writeln!(
            self.stdin,
            "go wtime {} btime {}",
            (wtime.as_secs_f64() * 1000.0).round() as u64,
            (btime.as_secs_f64() * 1000.0).round() as u64
        )?;

        let mut buffer = String::new();
        loop {
            buffer.clear();
            self.stdout.read_line(&mut buffer)?;
            println!("LINE: {}", buffer.trim());
            if buffer.starts_with("bestmove") {
                return Ok(UciMove::from_name(
                    buffer
                        .split_whitespace()
                        .skip(1)
                        .next()
                        .ok_or_else(|| anyhow!("Move missing after `bestmove` command"))?,
                    b,
                )
                .ok_or_else(|| anyhow!("invalid move"))?);
            }
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        writeln!(self.stdin, "quit").unwrap();
        self.child.wait().unwrap();
    }
}
pub fn play(
    white: &Path,
    black: &Path,
    start_fen: &str,
    time: f32,
    increment: Option<f32>,
) -> Result<GameOutcome> {
    let res = play_inner(white, black, start_fen, time, increment)?;
    println!(
        "PLAYED GAME: {} vs {}, with position {} => OUTCOME: {:?}",
        white.display(),
        black.display(),
        start_fen,
        res
    );
    Ok(res)
}

fn play_inner(
    white: &Path,
    black: &Path,
    start_fen: &str,
    time: f32,
    increment: Option<f32>,
) -> Result<GameOutcome> {
    let mut board = Board::from_fen(start_fen, EndChain)?;
    let mut boards = Vec::new();
    boards.push(board.clone());
    let mut moves_played = Vec::<UciMove>::new();

    let mut white = Engine::from_path(white)?;
    let mut black = Engine::from_path(black)?;

    let mut white_time = Duration::from_secs_f32(time);
    let mut black_time = Duration::from_secs_f32(time);

    let mov_gen = MoveGenerator::new();

    loop {
        println!("BOARD:\n{}", board);
        let info = mov_gen.gen_info(&board);
        if mov_gen.check_mate(&board, &info) {
            if board.state.player == chess_core::Player::White {
                return Ok(GameOutcome::Lost);
            } else {
                return Ok(GameOutcome::Won);
            }
        }
        if mov_gen.drawn(&board, &info) {
            return Ok(GameOutcome::Drawn);
        }

        let start = boards.len().saturating_sub(3);
        let end = boards
            .len()
            .saturating_sub(1)
            .saturating_sub(board.state.move_clock as usize);
        let mut rep_count = 0;
        for i in (end..=start).rev().step_by(2) {
            if boards[i].is_equal(&board) {
                rep_count += 1;
                if rep_count >= 3 {
                    return Ok(GameOutcome::Drawn);
                }
            }
        }

        if board.state.player == chess_core::Player::White {
            let time = Instant::now();
            let m = white.run(start_fen, &moves_played, &board, white_time, black_time)?;
            let elapsed = time.elapsed();
            if white_time < elapsed {
                return Ok(GameOutcome::Lost);
            }
            white_time -= elapsed;
            white_time += increment
                .map(Duration::from_secs_f32)
                .unwrap_or(Duration::ZERO);
            moves_played.push(m);
            board.make_move(m.0);
            boards.push(board.clone());
        } else {
            let time = Instant::now();
            let m = black.run(start_fen, &moves_played, &board, white_time, black_time)?;
            let elapsed = time.elapsed();
            if black_time < elapsed {
                return Ok(GameOutcome::Won);
            }
            black_time -= elapsed;
            black_time += increment
                .map(Duration::from_secs_f32)
                .unwrap_or(Duration::ZERO);
            moves_played.push(m);
            board.make_move(m.0);
            boards.push(board.clone());
        };
    }
}
