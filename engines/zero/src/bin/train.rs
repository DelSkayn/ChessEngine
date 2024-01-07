#![allow(dead_code)]

use candle_core::{DType, Device, Tensor};
use candle_nn::{loss::mse, AdamW, Optimizer, ParamsAdamW, VarBuilder, VarMap};
use clap::Parser;
use common::{misc::Outcome, Move, Promotion};
use engine_zero::board::{to_tensor, POSITION_TENSOR_LEN};
use file::{FileBoard, FromBytesReader, Game};
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use rand::{rngs::StdRng, Rng, SeedableRng};

static SHOULD_STOP: AtomicBool = AtomicBool::new(false);

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    games: PathBuf,
    #[arg(short, long)]
    model: Option<PathBuf>,
    #[arg(long, default_value_t = 64)]
    batch_size: usize,
    #[arg(long, default_value_t = 100)]
    game_sample_count: usize,
    #[arg(long)]
    load: Option<PathBuf>,
    #[arg(long)]
    skip_batches: Option<usize>,
    #[arg(long, short)]
    output: Option<PathBuf>,
}

struct PositionInfo {
    score: f32,
    board: FileBoard,
    m: Option<Move>,
}

pub struct GameBatcher<R: Read> {
    positions: Vec<Vec<PositionInfo>>,
    reader: FromBytesReader<Game, R>,
    random: StdRng,
    batch_size: usize,
}

pub struct TrainingBatch {
    input: Tensor,
    policy: Tensor,
    value: Tensor,
}

impl<R: Read> GameBatcher<R> {
    pub fn new(reader: R, parallel_games: usize, batch_size: usize) -> Result<Self> {
        let mut reader = FromBytesReader::new(reader);
        let mut positions = Vec::new();
        for _ in 0..parallel_games {
            if let Some(x) = Self::read_game(&mut reader) {
                let x = x?;
                positions.push(x);
            }
        }
        Ok(GameBatcher {
            positions,
            reader,
            random: StdRng::seed_from_u64(984749123123),
            batch_size,
        })
    }

    fn next_batch(&mut self, device: &Device) -> Result<Option<TrainingBatch>> {
        if self.positions.is_empty() {
            return Ok(None);
        }

        let mut buffer = [0u8; engine_zero::board::POSITION_TENSOR_LEN];

        let mut position_buffer = Vec::with_capacity(POSITION_TENSOR_LEN * self.batch_size);
        let mut policy_buffer = Vec::with_capacity((64 + 8) * 8 * 8);
        let mut scores = Vec::with_capacity(self.batch_size);

        let mut actual_size = self.batch_size;
        for i in 0..self.batch_size {
            if self.positions.is_empty() {
                actual_size = i;
                break;
            }

            let pick = self.random.gen_range(0..self.positions.len());
            let pick_inner = self.random.gen_range(0..self.positions[pick].len());
            let info = self.positions[pick].swap_remove(pick_inner);
            if self.positions[pick].is_empty() {
                if let Some(x) = Self::read_game(&mut self.reader) {
                    let x = x?;
                    self.positions[pick] = x;
                } else {
                    self.positions.swap_remove(pick);
                }
            }

            scores.push(info.score);

            to_tensor(&info.board.to_board(), &mut buffer);
            position_buffer.extend_from_slice(&buffer);

            let m = info.m.unwrap_or(Move::NULL);
            let from = m.from();
            let to = m.to();

            let mut rank = to.rank() as u32 * 8;
            if let Some(Promotion::Queen) = m.get_promotion() {
                rank += 8;
            };

            let target =
                rank * 8 * 8 + to.file() as u32 * 8 * 8 + from.rank() as u32 * 8 + to.file() as u32;

            policy_buffer.push(target);
        }

        let input = Tensor::from_vec(position_buffer, (actual_size, 13, 8, 8), device)?;
        let policy = Tensor::from_vec(policy_buffer, (actual_size,), device)?;
        let value = Tensor::from_vec(scores, (actual_size, 1), device)?;

        Ok(Some(TrainingBatch {
            input,
            policy,
            value,
        }))
    }

    fn read_game(r: &mut FromBytesReader<Game, R>) -> Option<Result<Vec<PositionInfo>>> {
        let game = match r.next()? {
            Ok(x) => x,
            Err(e) => return Some(Err(anyhow!(e))),
        };

        let mut res = Vec::new();

        let mut board = game.start_position.to_board();
        for m in game.moves {
            let score = match game.outcome {
                Outcome::Won { by, .. } => {
                    if by == board.state.player {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => 0.5,
            };

            res.push(PositionInfo {
                board: FileBoard::from_board(&board),
                score,
                m: Some(m),
            });
            board.make_move(m);
        }

        Some(Ok(res))
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    ctrlc::set_handler(|| {
        let res = SHOULD_STOP.swap(true, Ordering::AcqRel);
        if res {
            std::process::exit(0)
        }
    })
    .unwrap();

    let file = File::open(args.games)?;
    let mut reader = GameBatcher::new(file, args.game_sample_count, args.batch_size)?;

    let device = Device::cuda_if_available(0).unwrap();
    println!("using device: {device:?}");

    let (send, recv) = mpsc::sync_channel(4);

    let dev_clone = device.clone();
    std::thread::spawn(move || {
        if let Err(e) = (move || -> Result<()> {
            loop {
                let Some(batch) = reader.next_batch(&dev_clone)? else {
                    break;
                };
                if send.send(batch).is_err() {
                    break;
                };
            }
            Ok(())
        })() {
            eprintln!("error on load thread: {e}");
        }
    });

    let mut varmap = VarMap::new();
    if let Some(load) = args.load.as_ref() {
        varmap.load(load)?;
    };

    let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
    let mb = engine_zero::network::EfficientNet::new(vb).unwrap();

    if let Some(load) = args.load.as_ref() {
        varmap.load(load)?;
    };

    println!("parameters");
    let mut total_count = 0;
    for (k, v) in varmap.data().lock().unwrap().iter() {
        let count = v.shape().elem_count();
        total_count += count;
        println!("{k} = {}", count)
    }

    println!();
    println!("total parameters: {total_count}");

    let mut opt = AdamW::new(
        varmap.all_vars(),
        ParamsAdamW {
            lr: 0.0001,
            ..Default::default()
        },
    )?;

    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner} samples: {pos} @ {per_sec} loss: {msg} [elapsed: {elapsed}]",
        )
        .unwrap(),
    );

    let mut batch_count = 0;

    loop {
        let Ok(batch) = recv.recv() else {
            break;
        };

        if let Some(skip) = args.skip_batches {
            if batch_count < skip {
                batch_count += 1;
                bar.inc(args.batch_size as u64);
                continue;
            }
            if batch_count == skip {
                bar.reset_eta()
            }
        }

        let (policy, value) = mb.forward(&batch.input.to_dtype(DType::F32)?)?;

        let policy_prob = candle_nn::ops::log_softmax(&policy, 1)?.flatten_from(1)?;

        let policy_loss = candle_nn::loss::nll(&policy_prob, &batch.policy)?;
        let value_loss = mse(&value, &batch.value)?;

        let loss = value_loss.add(&policy_loss)?;
        opt.backward_step(&loss)?;

        bar.set_message(format!("{:.4}", loss.to_vec0::<f32>()?));
        bar.inc(args.batch_size as u64);

        if SHOULD_STOP.load(Ordering::Acquire) {
            break;
        }

        batch_count += 1;
    }

    bar.abandon();

    let model_path = args.output.unwrap_or_else(|| {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Path::new(&format!(
            "models/bc_{batch_count}_bs_{}_{:.4}.safetensor",
            args.batch_size, time
        ))
        .to_path_buf()
    });

    if let Some(parent) = model_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    varmap.save(model_path)?;

    Ok(())
}
