use std::mem::MaybeUninit;

use chess_core::{
    bb::BB,
    util::{BoardArray, PieceArray},
    Piece, Player, Square,
};
use sycamore::prelude::*;

#[derive(Clone, PartialEq)]
pub struct PieceInfo {
    id: u32,
    piece: Piece,
    square: RcSignal<Square>,
}

/*
impl PartialEq for PieceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
*/

#[derive(Clone)]
pub struct Position {
    next_id: u32,
    position: PieceArray<BB>,
    pieces: BoardArray<Option<PieceInfo>>,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Position {
    pub fn new() -> Self {
        let array = unsafe {
            let mut array = MaybeUninit::<[Option<PieceInfo>; 64]>::uninit();
            for i in 0..64 {
                array
                    .as_mut_ptr()
                    .cast::<Option<PieceInfo>>()
                    .add(i)
                    .write(None);
            }
            array.assume_init()
        };

        Position {
            next_id: 0,
            position: PieceArray::new(BB::empty()),
            pieces: BoardArray::new_array(array),
        }
    }

    pub fn update_to(mut self, position: PieceArray<BB>) -> Self {
        let mut new_pieces = self.pieces.clone();

        for piece in Piece::WhiteKing.to(Piece::BlackPawn) {
            if self.position[piece] == position[piece] {
                continue;
            }
            let from = self.position[piece] & !position[piece];
            let to = position[piece] & !self.position[piece];
            let mut from = from.iter();
            let mut to = to.iter();

            loop {
                let pair = (from.next(), to.next());
                match pair {
                    (None, None) => break,
                    (None, Some(t)) => {
                        let id = self.next_id;
                        self.next_id = self.next_id.wrapping_add(1);
                        let square = create_rc_signal(t);
                        new_pieces[t] = Some(PieceInfo { id, piece, square })
                    }
                    (Some(f), None) => {
                        new_pieces[f] = None;
                    }
                    (Some(f), Some(t)) => {
                        if let Some(p) = self.pieces[f].take() {
                            p.square.set(t);
                            new_pieces[t] = Some(p);
                        } else {
                            let id = self.next_id;
                            self.next_id = self.next_id.wrapping_add(1);
                            let square = create_rc_signal(t);
                            new_pieces[t] = Some(PieceInfo { id, piece, square })
                        }
                    }
                }
            }
        }
        self.position = position;
        self.pieces = new_pieces;

        self
    }
}

pub struct BoardContext {
    pub position: RcSignal<Position>,
    pub view: RcSignal<Player>,
}

impl BoardContext {
    pub fn new() -> BoardContext {
        Self {
            position: create_rc_signal(Position::new()),
            view: create_rc_signal(Player::White),
        }
    }

    pub fn from_position(position: PieceArray<BB>) -> BoardContext {
        Self {
            position: create_rc_signal(Position::new().update_to(position)),
            view: create_rc_signal(Player::White),
        }
    }
}

#[component(inline_props)]
pub fn Piece<G: Html>(cx: Scope, view: RcSignal<Player>, info: PieceInfo) -> View<G> {
    static PIECE_SRC_MAP: [&str; 12] = [
        "wK.svg", "wQ.svg", "wB.svg", "wN.svg", "wR.svg", "wP.svg", "bK.svg", "bQ.svg", "bB.svg",
        "bN.svg", "bR.svg", "bP.svg",
    ];

    let square = info.square;
    let style = create_memo(cx, move || {
        let sq = square.get();
        let white = *view.get() == Player::White;

        let (file, rank) = sq.to_file_rank();

        let file_proc = if white {
            100.0 / 8.0 * file as f32
        } else {
            100.0 - 100.0 / 8.0 * (file + 1) as f32
        };
        let rank_proc = if white {
            100.0 / 8.0 * rank as f32
        } else {
            100.0 - 100.0 / 8.0 * (rank + 1) as f32
        };

        format!("bottom: {rank_proc}%; left: {file_proc}%")
    });

    let piece_src = PIECE_SRC_MAP[info.piece as usize];
    let src = format!("pieces/{piece_src}");

    view! {cx,
        img(style=style,
            src=src,
            class="w-1/8 h-1/8 absolute transition-all duration-200 ease-in-out select-none z-0")
    }
}

#[component]
pub fn Board<G: Html>(cx: Scope) -> View<G> {
    let ctx = use_context::<BoardContext>(cx);
    let position = &ctx.position;
    let view = &ctx.view;

    let pieces = create_memo(cx, || {
        position
            .get()
            .pieces
            .iter()
            .filter_map(|x| x.as_ref())
            .cloned()
            .collect::<Vec<_>>()
    });

    view! { cx,
        div(class="w-full relative aspect-square pointer-events-none rounded overflow-hidden z-0 shadow"){
            img(draggable="false", src="board/gray.svg", alt="board", class="w-full h-full static select-none z-0"){}
            Keyed(
                iterable=pieces,
                view= move |cx, p| view!{ cx,
                    Piece(view=view.clone(),info=p)
                },
                key = |x| x.id
            )
        }
    }
}
