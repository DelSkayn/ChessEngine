use chess_core::{util::BoardArray, Board, Piece, Square};
use seed::{attrs, div, img, prelude::*, C};

pub struct Model {
    next_id: usize,
    pub board: Board,
    pieces: BoardArray<Option<(Piece, usize)>>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            next_id: 0,
            board: Board::empty(),
            pieces: BoardArray::new(None),
        }
    }

    pub fn set(&mut self, board: Board) {
        for p in Piece::WhiteKing.to(Piece::BlackPawn) {
            let from = self.board.pieces[p] & !board.pieces[p];
            let to = board.pieces[p] & !self.board.pieces[p];

            let mut from_iter = from.iter();
            let mut to_iter = to.iter();

            loop {
                match (from_iter.next(), to_iter.next()) {
                    (None, None) => break,
                    (Some(f), Some(t)) => {
                        self.pieces[t] = self.pieces[f];
                        self.pieces[f] = None;
                    }
                    (Some(f), None) => {
                        if self.pieces[f].map(|x| x.0) == Some(p) {
                            self.pieces[f] = None;
                        }
                    }
                    (None, Some(t)) => {
                        self.pieces[t] = Some((p, self.next_id));
                        self.next_id = self.next_id.wrapping_add(1);
                    }
                }
            }
        }
        self.board = board;
    }
}

pub fn view(model: &Model, classes: &str) -> Node<super::Msg> {
    div![
        C![classes],
        div![
            C!["w-full h-full relative aspect-square pointer-events-none rounded overflow-hidden"],
            img![
                C!["w-full h-full static select-none"],
                attrs! {
                    At::Draggable => "false",
                    At::Src => "board/gray.svg",
                    At::Alt => "board",
                }
            ],
            (0..64)
                .map(Square::new)
                .filter_map(|s| model.pieces[s].clone().map(|x| (s, x)))
                .map(|(square, (kind, id))| { view_piece(kind, square, id) })
        ]
    ]
}

fn view_piece(kind: Piece, square: Square, id: usize) -> Node<super::Msg> {
    const PIECE_SRC_MAP: [&'static str; 12] = [
        "wK.svg", "wQ.svg", "wB.svg", "wN.svg", "wR.svg", "wP.svg", "bK.svg", "bQ.svg", "bB.svg",
        "bN.svg", "bR.svg", "bP.svg",
    ];

    let piece_src = PIECE_SRC_MAP[kind as usize];
    let (file, rank) = square.to_file_rank();

    let file_proc = 100.0 / 8.0 * file as f32;
    let rank_proc = 100.0 / 8.0 * rank as f32;

    img![
        el_key(&id),
        C!["w-1/8 h-1/8 absolute transition-all ease-in-out select-none"],
        attrs! {
            At::Style => format!("bottom: {rank_proc}%; left: {file_proc}%")
            At::Src => format!("pieces/{piece_src}")
        }
    ]
}
