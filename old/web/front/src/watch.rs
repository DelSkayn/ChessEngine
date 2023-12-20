use std::time::Duration;

use crate::board::{Board, BoardContext};
use chess_core::{board::EndChain, Board as ChessBoard};
use common::game;
use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message, WebSocketError};
use gloo_timers::future::sleep;
use log::{error, info};
use sycamore::{futures::spawn_local_scoped, prelude::*};

#[component]
pub fn Watch<G: Html>(cx: Scope) -> View<G> {
    let board = BoardContext::from_position(ChessBoard::start_position(EndChain).pieces);
    let _board = provide_context(cx, board);

    spawn_local_scoped(cx, async {
        loop {
            let host = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .location()
                .unwrap()
                .host()
                .unwrap();
            let mut ws = match WebSocket::open(&format!("ws://{host}/api/ws")) {
                Ok(x) => x,
                Err(e) => {
                    error!("creating game socket: {e}");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            while let Some(m) = ws.next().await {
                let m = match m {
                    Ok(x) => x,
                    Err(WebSocketError::ConnectionError) => {
                        error!("connecting to game socket");
                        continue;
                    }
                    Err(WebSocketError::ConnectionClose(e)) => {
                        error!("connection to game socket closed: {:?}", e);
                        continue;
                    }
                    Err(WebSocketError::MessageSendError(e)) => {
                        error!("failed to send message: {e}");
                        continue;
                    }
                    _ => continue,
                };

                let text = match m {
                    Message::Text(x) => x,
                    Message::Bytes(_) => {
                        error!("recieved binary message from websocket");
                        continue;
                    }
                };

                let m = serde_json::from_str::<game::Event>(&text);
                info!("message: {:?}", m);
            }
            error!("web socket disconnected");
            sleep(Duration::from_secs(1)).await;
        }
    });

    view! { cx,
        div(class="container h-full flex p-2"){
            div(class="w-96"){
                Board{}
            }
        }
    }
}
