use crate::{
    board::{Board, BoardContext},
    components::ErrorAlert,
};
use chess_core::{board::EndChain, Board as ChessBoard};
use log::info;
use sycamore::prelude::*;

#[component]
pub fn Position<G: Html>(cx: Scope) -> View<G> {
    let ctx = BoardContext::from_position(ChessBoard::start_position(EndChain).pieces);
    let ctx = provide_context(cx, ctx);

    let fen = create_signal(cx, String::new());
    let name = create_signal(cx, String::new());
    let error = create_signal::<Option<String>>(cx, None);
    let fen_valid = create_signal(cx, false);

    let disabled = create_memo(cx, || !(*fen_valid.get()) || name.get().is_empty());

    create_effect(cx, || {
        let fen = fen.get();

        if error.get_untracked().is_some() {
            error.set(None);
        }

        if fen.is_empty() {
            fen_valid.set(false);
            ctx.position.set(
                (*ctx.position.get_untracked())
                    .clone()
                    .update_to(ChessBoard::start_position(EndChain).pieces),
            );
            return;
        }

        let b = match ChessBoard::from_fen(&fen, EndChain) {
            Ok(x) => x,
            Err(e) => {
                info!("bla {}", fen);
                fen_valid.set(false);
                error.set(Some(format!("Invalid fen: {}", e)));
                return;
            }
        };
        fen_valid.set(true);

        ctx.position
            .set((*ctx.position.get_untracked()).clone().update_to(b.pieces));
    });

    view! { cx,
        div(class="container h-full flex flex-col p-2"){
            div(class="w-96 h-96"){
                Board{}
            }
            div(class="py-4"){
                    input(type="input", placeholder="Name", class="input input-bordered mb-2 ", bind:value=name){}
                    input(type="input", placeholder="Fen string", class="input input-bordered w-full mb-2", bind:value=fen){}
                    button(class="btn btn-primary",disabled=*disabled.get()){ "Create"}
            }
            ErrorAlert(error=error)
        }
    }
}
