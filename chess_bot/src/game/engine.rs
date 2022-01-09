use std::{path::Path, process::Stdio, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use chess_core::{
    board::EndChain,
    gen::{gen_type, MoveGenerator},
    Board, Player, Square,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};

pub struct Engine {
    board: Board,
    move_gen: MoveGenerator,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    running: bool,
}

impl Engine {
    pub fn new(path: &Path) -> Result<Self> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Could not run engine command")?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Missing stdin from child process"))
            .unwrap();

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Missing stdin from child process"))
            .unwrap();

        Ok(Engine {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            board: Board::start_position(EndChain),
            move_gen: MoveGenerator::new(),
            running: false,
        })
    }

    async fn start_uci(stdin: &mut ChildStdin, stdout: &mut BufReader<ChildStdout>) {
        let mut stdout = BufReader::new(stdout);
        stdin.write_all(b"uci\n").await.unwrap();
        let mut line_buffer = String::new();
        loop {
            line_buffer.clear();
            stdout.read_line(&mut line_buffer).await.unwrap();
            trace!("engine out: {}", line_buffer.trim());
            if line_buffer.trim() == "uciok" {
                return;
            }
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        tokio::time::timeout(
            Duration::from_secs(2),
            Self::start_uci(&mut self.stdin, &mut self.stdout),
        )
        .await
        .context("Uci startup timedout")
    }

    pub async fn set_position(&mut self, fen: &str, moves: &str) -> Result<()> {
        let mut pos_string = String::new();
        pos_string.push_str("position ");
        if fen == "startpos" {
            pos_string.push_str("startpos");
            self.board = Board::start_position(EndChain);
        } else {
            pos_string.push_str("fen");
            pos_string.push_str(fen);
            pos_string.push(' ');
            self.board = Board::from_fen(fen, EndChain)?;
        }
        if !moves.is_empty() {
            pos_string.push_str(" moves ");
            pos_string.push_str(moves);

            let mut moves_board = Vec::new();

            for m in moves.split_whitespace() {
                moves_board.clear();
                self.move_gen
                    .gen_moves::<gen_type::All, _, _>(&self.board, &mut moves_board);
                let from =
                    Square::from_name(&m[..2]).ok_or_else(|| anyhow!("invalid move name"))?;
                let to = Square::from_name(&m[2..4]).ok_or_else(|| anyhow!("invalid move name"))?;

                let m = moves_board
                    .iter()
                    .copied()
                    .find(|m| m.from() == from && m.to() == to)
                    .ok_or_else(|| anyhow!("Could not find move"))?;

                self.board.make_move(m);
            }
        }
        pos_string.push('\n');
        self.stdin.write_all(pos_string.as_bytes()).await?;
        Ok(())
    }

    pub async fn go(&mut self, wtime: u64, btime: u64) -> Result<()> {
        let cmd = format!("go wtime {} btime {}\n", wtime, btime);

        self.stdin.write_all(cmd.as_bytes()).await?;
        self.running = true;
        Ok(())
    }

    pub async fn get_move(&mut self) -> Result<String> {
        if !self.running {
            bail!("Engine is not running")
        }
        let mut buffer = String::new();
        loop {
            buffer.clear();
            self.stdout.read_line(&mut buffer).await?;
            trace!("engine: {}", buffer);
            if buffer.starts_with("bestmove") {
                let m = buffer
                    .split_whitespace()
                    .skip(1)
                    .next()
                    .ok_or_else(|| anyhow!("Missing move after `bestmove` command"))?;

                self.running = false;

                return Ok(m.to_string());
            }
        }
    }

    pub async fn quit(mut self) {
        async {
            self.stdin
                .write_all(b"quit\n")
                .await
                .context("Failed to write to process stdin")
                .map_err(|e| error!("Failed to stop engine process: {:?}", e))
                .ok();

            match tokio::time::timeout(Duration::from_secs(1), self.child.wait()).await {
                Ok(x) => {
                    x.context("Failed to wait on child")?;
                }
                Err(_) => {
                    warn!("Engine process failed to stop, killing process");
                    self.child.kill().await?;
                }
            };
            Result::<(), anyhow::Error>::Ok(())
        }
        .await
        .map_err(|e| error!("Failed to stop engine process: {:?}", e))
        .ok();
    }

    pub fn cur_player(&self) -> Player {
        self.board.state.player
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}
