use std::{collections::VecDeque, sync::Arc};

use common::game::{self, NoContestReason, Outcome};
use tokio::{
    select,
    sync::{broadcast, mpsc, oneshot, Notify, RwLock},
    task::JoinHandle,
};
use tracing::{error, warn};

use super::{game as server_game, ScheduledGame};

pub struct GameSubscription {
    pub catch_up: Vec<game::Event>,
    pub events: broadcast::Receiver<game::Event>,
}

pub struct Colosseum {
    scheduled_games: Arc<RwLock<VecDeque<ScheduledGame>>>,
    catch_up: RwLock<Vec<game::Event>>,
    subscribe: mpsc::Sender<oneshot::Sender<GameSubscription>>,
    notify: Arc<Notify>,
    play_handle: JoinHandle<()>,
}

impl Drop for Colosseum {
    fn drop(&mut self) {
        self.play_handle.abort();
    }
}

impl Colosseum {
    pub fn new() -> Self {
        let (send, recv) = broadcast::channel(32);
        let (sub, publ) = mpsc::channel(32);

        tokio::spawn(Self::handle_subscriptions(recv, publ));

        let scheduled_games = Arc::new(RwLock::new(VecDeque::new()));
        let notify = Arc::new(Notify::new());

        let play_handle = tokio::spawn(Self::play_game(
            send,
            scheduled_games.clone(),
            notify.clone(),
        ));

        Colosseum {
            scheduled_games,
            catch_up: RwLock::new(Vec::new()),
            subscribe: sub,
            notify,
            play_handle,
        }
    }

    async fn play_game(
        mut sender: broadcast::Sender<game::Event>,
        games: Arc<RwLock<VecDeque<ScheduledGame>>>,
        notify: Arc<Notify>,
    ) {
        loop {
            while let Some(g) = games.write().await.pop_back() {
                match server_game::play(&g, &mut sender).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("failed to run game: {e}");
                        sender
                            .send(game::Event::GameEnded {
                                elo_gain_white: 0.0,
                                elo_gain_black: 0.0,
                                updated_elo_white: g.white.elo,
                                updated_elo_black: g.black.elo,
                                outcome: Outcome::NoContest(NoContestReason::EngineCrashed),
                            })
                            .ok();
                    }
                }
            }
            notify.notified().await;
        }
    }

    async fn handle_subscriptions(
        mut events: broadcast::Receiver<game::Event>,
        mut new_subscriptions: mpsc::Receiver<oneshot::Sender<GameSubscription>>,
    ) {
        let mut catch_up = Vec::new();
        loop {
            select! {
                subscription = new_subscriptions.recv() => {
                    match subscription{
                        Some(x) => {
                            let new_events = events.resubscribe();
                            let catch_up = catch_up.clone();
                            x.send(GameSubscription{
                                events: new_events,
                                catch_up
                            }).ok();
                        }
                        None => {
                            break
                        }
                    }
                }
                event = events.recv() => {
                    match event {
                        Ok(event) => {
                            match event{
                                game::Event::GameEnded{ .. } => {
                                    catch_up.clear();
                                }
                                event => {
                                    catch_up.push(event);
                                }
                            }
                        },
                        Err(broadcast::error::RecvError::Closed) => {
                            break
                        }
                        Err(broadcast::error::RecvError::Lagged(x)) => {
                            warn!("subscription task lagged behind: skipped {x} messages");
                        }
                    }
                }
            }
        }
    }

    pub async fn subscribe(&self) -> GameSubscription {
        let (send, recv) = oneshot::channel();
        self.subscribe
            .send(send)
            .await
            .map_err(|_| ())
            .expect("subscription manage task quit unexpectedly");
        recv.await
            .expect("subscription manage task quit unexpectedly")
    }

    pub async fn schedule_games(&self, g: Vec<ScheduledGame>) {
        let mut sched = self.scheduled_games.write().await;
        for game in g {
            sched.push_front(game);
        }
        self.notify.notify_one();
    }

    pub async fn get_scheduled_games(&self) -> Vec<ScheduledGame> {
        self.scheduled_games.read().await.iter().cloned().collect()
    }
}
