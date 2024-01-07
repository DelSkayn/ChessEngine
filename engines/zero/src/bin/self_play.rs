use anyhow::Result;
use clap::Parser;
use common::{
    board::Board,
    misc::{DrawCause, Outcome, WinCause},
};
use file::{FileBoard, Game, ToBytes};
use indicatif::{ProgressBar, ProgressStyle};
use move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use rand::seq::SliceRandom;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::UNIX_EPOCH,
};

static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    random: bool,
    #[arg(short, long)]
    output: Option<String>,
    #[arg(short, long)]
    limit: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    ctrlc::set_handler(|| {
        let res = SHOULD_QUIT.swap(true, Ordering::Relaxed);
        if res {
            std::process::exit(0)
        }
    })
    .unwrap();

    let file_path = args.output.unwrap_or_else(|| {
        let res = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        format!("games/{:.4}.games", res)
    });
    let file_path = Path::new(&file_path);

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("writing to `{}`", file_path.display());

    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(file_path)?;

    let mut bar = if let Some(limit) = args.limit {
        let bar = ProgressBar::new(limit as u64);
        bar.set_style(
            ProgressStyle::with_template("[{elapsed}/{duration}] {wide_bar} {pos}/{len} {per_sec}")
                .unwrap(),
        );
        bar
    } else {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner} games played: {pos} @ {per_sec} [elapsed: {elapsed}]",
            )
            .unwrap(),
        );
        bar
    };

    if args.random {
        random_play(&mut file, &mut bar, args.limit)?;
    } else {
        anyhow::bail!("non random play not yet implemented")
    }

    Ok(())
}

fn random_play(file: &mut File, progress: &mut ProgressBar, limit: Option<usize>) -> Result<()> {
    let mut writer = BufWriter::new(file);

    let gen = MoveGenerator::new();
    let mut game = Game {
        start_position: FileBoard::from_board(&Board::start_position()),
        moves: Vec::new(),
        outcome: Outcome::None,
    };
    let mut played_hash = Vec::new();
    let mut move_buffer = InlineBuffer::new();

    let mut games_played = 0;

    loop {
        if let Some(limit) = limit {
            if games_played >= limit {
                break;
            }
        }

        played_hash.clear();
        game.moves.clear();

        let mut board = Board::start_position();
        played_hash.push(board.hash);

        'main: loop {
            move_buffer.clear();
            let info = gen.gen_moves::<gen_type::All>(&board, &mut move_buffer);

            if gen.drawn_by_rule(&board, &info) {
                game.outcome = Outcome::Drawn(DrawCause::Material);
                break;
            }

            if move_buffer.is_empty() {
                if gen.checked_king(&board, &info) {
                    game.outcome = Outcome::Won {
                        by: board.state.player.flip(),
                        cause: WinCause::Mate,
                    };
                } else {
                    game.outcome = Outcome::Drawn(DrawCause::Stalemate);
                }
                break;
            }

            for m in move_buffer.iter() {
                let undo = board.make_move(m);
                let info = gen.gen_info(&board);
                if gen.check_mate(&board, &info) {
                    game.moves.push(m);
                    played_hash.push(board.hash);
                    continue 'main;
                }
                board.unmake_move(undo);
            }

            let mut count = 0;
            loop {
                let m = move_buffer
                    .as_slice()
                    .choose(&mut rand::thread_rng())
                    .unwrap();

                let undo = board.make_move(*m);
                if !played_hash.contains(&board.hash) {
                    game.moves.push(*m);
                    played_hash.push(board.hash);
                    break;
                }
                count += 1;
                if count > 10 {
                    game.outcome = Outcome::Drawn(DrawCause::Repetition);
                    break;
                }
                board.unmake_move(undo)
            }

            if board.state.move_clock > 50 {
                game.outcome = Outcome::Drawn(DrawCause::FiftyMove);
                break;
            }
        }

        game.to_bytes(&mut writer)?;

        if SHOULD_QUIT.load(Ordering::Relaxed) {
            break;
        }

        progress.inc(1);
        games_played += 1;
    }

    writer.flush()?;
    Ok(())
}
