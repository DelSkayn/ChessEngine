use std::{
    path::Path,
    time::{Duration, Instant},
};

use crate::codec::uci::EngineCodec;

use super::{sandbox::Sandbox, ScheduledGame};
use chess_core::{
    board::EndChain,
    gen::MoveGenerator,
    uci::{Cmd, GoCmd, Msg, PositionCmd, UciMove},
    Board, GameOutcome, Move, Player,
};
use common::game::{self, WonBy};
use futures::{SinkExt, StreamExt};
use tokio::{sync::broadcast, time::timeout};
use tokio_util::codec::Framed;
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("tried to play a game on an invalid fen string: `{0}`")]
    InvalidPosition(String),
    #[error("failed to start engine for player: {0:?}")]
    EngineFailed(Player),
    #[error("engine for player `{0:?}` quit unexpectedly")]
    EngineQuit(Player),
    #[error("engine failed to initialize within timeout for player: {0:?}")]
    InitializeTimeout(Player),
    #[error("engine returned an invalid move for the current position for player: {0:?}")]
    InvalidMove(Player),
}

pub async fn wait_for_uciok<T: StreamExt<Item = Result<Msg, std::io::Error>> + Unpin>(
    stream: &mut T,
) -> Result<(), ()> {
    while let Some(x) = stream.next().await {
        match x {
            Ok(Msg::UciOk) => return Ok(()),
            Err(e) => {
                error!("error reading from engine: {e}");
            }
            _ => {}
        }
    }
    return Err(());
}

pub async fn play(
    game: &ScheduledGame,
    sender: &mut broadcast::Sender<game::Event>,
) -> Result<game::Outcome, Error> {
    let gen = MoveGenerator::new();

    let Ok(mut board) = Board::from_fen(&game.position, EndChain) else{
        return Err(Error::InvalidPosition(game.position.clone()))
    };

    let white_file = &game.white.engine_file;
    let white_sandbox =
        Sandbox::from_executable_path(&Path::new("./engines").join(white_file)).await;
    let white_sandbox = match white_sandbox {
        Ok(x) => x,
        Err(e) => {
            error!("failed to start white engine: {e}");
            return Err(Error::EngineFailed(Player::White));
        }
    };

    let black_file = &game.black.engine_file;
    let black_sandbox =
        Sandbox::from_executable_path(&Path::new("./engines").join(black_file)).await;
    let black_sandbox = match black_sandbox {
        Ok(x) => x,
        Err(e) => {
            error!("failed to start black engine: {e}");
            return Err(Error::EngineFailed(Player::Black));
        }
    };

    let mut white_sandbox = Framed::new(white_sandbox, EngineCodec);
    let mut black_sandbox = Framed::new(black_sandbox, EngineCodec);

    let mut white_time = game.time.clone();
    let mut black_time = game.time.clone();

    white_sandbox
        .send(Cmd::Uci)
        .await
        .map_err(|_| Error::EngineQuit(Player::White))?;
    match timeout(Duration::from_secs(2), wait_for_uciok(&mut white_sandbox)).await {
        Ok(Ok(_)) => {}
        Ok(Err(_)) => return Err(Error::EngineQuit(Player::White)),
        Err(_) => return Err(Error::InitializeTimeout(Player::White)),
    }

    black_sandbox
        .send(Cmd::Uci)
        .await
        .map_err(|_| Error::EngineQuit(Player::Black))?;
    match timeout(Duration::from_secs(2), wait_for_uciok(&mut black_sandbox)).await {
        Ok(Ok(_)) => {}
        Ok(Err(_)) => return Err(Error::EngineQuit(Player::Black)),
        Err(_) => return Err(Error::InitializeTimeout(Player::Black)),
    }

    sender
        .send(game::Event::StartGame {
            position: game.position.clone(),
            white: common::engine::Engine::from(game.white.clone()),
            black: common::engine::Engine::from(game.black.clone()),
        })
        .ok();

    let mut moves = Vec::<Move>::new();

    let outcome = loop {
        let player = board.state.player;

        match GameOutcome::from_board(&board, &gen) {
            GameOutcome::None => {}
            GameOutcome::Won(x) => {
                break game::Outcome::Won(x, game::WonBy::Mate);
            }
            GameOutcome::Drawn => {
                break game::Outcome::Drawn;
            }
        }

        let mut cmd = vec![GoCmd::WhiteTime(white_time), GoCmd::BlackTime(black_time)];

        sender
            .send(game::Event::Time {
                white: white_time,
                black: black_time,
            })
            .ok();

        let (engine, time) = match player {
            Player::White => (&mut white_sandbox, &mut white_time),
            Player::Black => (&mut black_sandbox, &mut black_time),
        };

        if let Some(x) = game.increment.clone() {
            cmd.push(GoCmd::WhiteInc(x));
            cmd.push(GoCmd::BlackInc(x));
        }

        engine
            .send(Cmd::Position(PositionCmd {
                position: chess_core::uci::StartOrFen::Fen(game.position.clone()),
                moves: moves.iter().copied().map(UciMove::from).collect(),
            }))
            .await
            .map_err(|_| Error::EngineQuit(player))?;
        engine
            .send(Cmd::Go(cmd))
            .await
            .map_err(|_| Error::EngineQuit(player))?;

        let move_start = Instant::now();

        let m = loop {
            let msg = engine.next().await.ok_or(Error::EngineQuit(player))?;
            match msg {
                Err(e) => {
                    warn!("error reading from engine: {e}");
                }
                Ok(Msg::Info(x)) => {
                    for i in x {
                        sender.send(game::Event::Eval(player, i)).ok();
                    }
                }
                Ok(Msg::BestMove { r#move, .. }) => {
                    break r#move;
                }
                Ok(_) => {}
            }
        };

        let time_taken = move_start.elapsed();
        *time = match time.checked_sub(time_taken) {
            Some(x) => x,
            None => {
                break game::Outcome::Won(player.flip(), WonBy::Timeout);
            }
        };

        if let Some(inc) = game.increment {
            *time += inc;
        }

        let Some(m) = m.to_move(&gen, &board) else {
            return Err(Error::InvalidMove(player))
        };

        moves.push(m);
        board.make_move(m);
        sender.send(game::Event::Move(m)).ok();
    };

    tokio::spawn(async move {
        white_sandbox.send(Cmd::Quit).await.ok();
        white_sandbox.into_inner().stop().await.ok();

        black_sandbox.send(Cmd::Quit).await.ok();
        black_sandbox.into_inner().stop().await.ok();
    });

    Ok(outcome)
}
