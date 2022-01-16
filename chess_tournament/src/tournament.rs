use anyhow::Result;
use rand::{thread_rng, Rng};

use crate::{
    elo::{self},
    game, Color, Config, GameOutcome, State,
};

pub fn start(config: &Config, state: &mut State) -> Result<()> {
    if state.0.len() < 2 {
        return Ok(());
    }

    let mut schedule = Vec::new();

    for (idx, e) in state.0.iter().enumerate() {
        for _ in e.games.len()..(config.initial_games / 2) {
            schedule.push(idx);
        }
    }

    for idx in 0..state.0.len() {
        for _ in 0..config.tournament_games {
            schedule.push(idx);
        }
    }

    for g in schedule {
        let other = match_make(state, g);
        let pos = thread_rng().gen_range(0..config.start_positions.len());
        println!(
            "SCHEDULED: {}(elo: {}) vs {}(elo: {}) on {}",
            state.0[g].path.display(),
            state.0[g].elo,
            state.0[other].path.display(),
            state.0[other].elo,
            config.start_positions[pos].name
        );
        play_game(config, state, g, other, pos)?;
    }

    Ok(())
}

pub fn match_make(state: &mut State, current: usize) -> usize {
    let cur_rating = state.0[current].elo;
    let mut picks: Vec<(usize, f64)> = state
        .0
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != current)
        .map(|(idx, e)| {
            let score = 0.5 - (0.5 - elo::outcome(cur_rating, e.elo)).abs();

            (idx, score)
        })
        .collect();

    picks.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());

    dbg!(&picks);

    let total: f64 = picks.iter().map(|e| e.1).sum();
    let mut pick = rand::thread_rng().gen_range(0.0..total);
    for (idx, s) in picks.iter() {
        dbg!(pick, s);
        pick -= s;
        if pick <= 0.0 {
            return *idx;
        }
    }
    unreachable!();
}

pub fn ref_mut_two<T>(s: &mut [T], first: usize, second: usize) -> (&mut T, &mut T) {
    assert!(first != second);
    if first < second {
        let (a, b) = s.split_at_mut(second);
        (&mut a[first], &mut b[0])
    } else {
        let (a, b) = s.split_at_mut(first);
        (&mut b[0], &mut a[second])
    }
}

pub fn update_elo(first: &mut f64, second: &mut f64, outcome: GameOutcome, k: f64) {
    let first_back = *first;

    elo::update(first, outcome.score(), *second, k);
    elo::update(second, outcome.flip().score(), first_back, k);
}

pub fn play_game(
    config: &Config,
    state: &mut State,
    first: usize,
    second: usize,
    position: usize,
) -> Result<()> {
    let (first, second) = ref_mut_two(&mut state.0, first, second);

    let outcome = game::play(
        &first.path,
        &second.path,
        &config.start_positions[position].fen,
        config.time,
        config.increment,
    )?;

    update_elo(
        &mut first.elo,
        &mut second.elo,
        outcome,
        config.k_factor as f64,
    );

    first.games.push(crate::GamePlayed {
        outcome,
        opponent: second.path.clone(),
        color: Color::White,
        start_position: config.start_positions[position].name.clone(),
    });

    second.games.push(crate::GamePlayed {
        outcome: outcome.flip(),
        opponent: first.path.clone(),
        color: Color::Black,
        start_position: config.start_positions[position].name.clone(),
    });

    let outcome = game::play(
        &second.path,
        &first.path,
        &config.start_positions[position].fen,
        config.time,
        config.increment,
    )?;

    update_elo(
        &mut first.elo,
        &mut second.elo,
        outcome.flip(),
        config.k_factor as f64,
    );
    second.games.push(crate::GamePlayed {
        outcome,
        opponent: first.path.clone(),
        color: Color::White,
        start_position: config.start_positions[position].name.clone(),
    });
    first.games.push(crate::GamePlayed {
        outcome: outcome.flip(),
        opponent: second.path.clone(),
        color: Color::Black,
        start_position: config.start_positions[position].name.clone(),
    });

    Ok(())
}
