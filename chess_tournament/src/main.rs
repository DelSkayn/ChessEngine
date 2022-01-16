use std::{
    fs::{self, File},
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chess_core::Player;
use serde::{Deserialize, Serialize};

mod elo;
mod game;
mod tournament;

#[derive(Deserialize, Serialize)]
pub struct Config {
    k_factor: f32,
    initial_games: usize,
    tournament_games: usize,
    start_positions: Vec<StartPosition>,
    time: f32,
    increment: Option<f32>,
}

#[derive(Deserialize, Serialize)]
pub struct StartPosition {
    name: String,
    fen: String,
}

#[derive(Deserialize, Serialize)]
pub struct State(Vec<EngineData>);

#[derive(Deserialize, Serialize)]
pub enum Color {
    White,
    Black,
}

impl From<Player> for Color {
    fn from(p: Player) -> Self {
        match p {
            Player::White => Color::White,
            Player::Black => Color::Black,
        }
    }
}

impl From<Color> for Player {
    fn from(p: Color) -> Self {
        match p {
            Color::White => Player::White,
            Color::Black => Player::Black,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct GamePlayed {
    outcome: GameOutcome,
    opponent: PathBuf,
    color: Color,
    start_position: String,
}

#[derive(Deserialize, Serialize, Clone, Copy, Eq, PartialEq, Debug)]
pub enum GameOutcome {
    Won,
    Lost,
    Drawn,
}

impl GameOutcome {
    pub fn flip(self) -> Self {
        match self {
            GameOutcome::Won => GameOutcome::Lost,
            GameOutcome::Lost => GameOutcome::Won,
            GameOutcome::Drawn => GameOutcome::Drawn,
        }
    }

    pub fn score(self) -> f64 {
        match self {
            GameOutcome::Won => 1.0,
            GameOutcome::Drawn => 0.5,
            GameOutcome::Lost => 0.0,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct EngineData {
    path: PathBuf,
    elo: f64,
    games: Vec<GamePlayed>,
}

fn main() -> Result<()> {
    let config_file = File::open("./config.json").context("Could not find config json")?;
    let config = serde_json::from_reader(config_file).context("Failed to parse config file")?;

    let mut state: State = if !Path::new("./state.json").exists() {
        State(Vec::new())
    } else {
        let state = File::open("./state.json").context("Could not open state file")?;
        serde_json::from_reader(state).context("Could not parse state file")?
    };

    update_state(&mut state)?;
    tournament::start(&config, &mut state)?;

    state
        .0
        .sort_unstable_by(|a, b| a.elo.partial_cmp(&b.elo).unwrap().reverse());

    let file = File::create("./state.json").context("could not write to state file")?;
    serde_json::to_writer_pretty(file, &state).context("Could not serialize tournament state")?;
    Ok(())
}

fn update_state(s: &mut State) -> Result<()> {
    for entry in fs::read_dir("./engines")? {
        let entry = entry?;
        // Test permission
        if entry.metadata()?.permissions().mode() & 0o111 == 0 {
            continue;
        }

        let path = entry.path();

        // Test if engine pressent in state;
        if s.0.iter().find(|x| x.path == path).is_none() {
            s.0.push(EngineData {
                path,
                elo: 1500.0,
                games: Vec::new(),
            });
        }
    }
    Ok(())
}
