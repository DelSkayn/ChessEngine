use chess_core::{board::EndChain, Board};
use seed::{div, p, prelude::*, C};

use crate::board;

#[derive(Debug)]
pub enum Msg {
    Board,
}

pub struct Model {
    board: board::Model,
    socket: Option<WebSocket>,
}

impl Model {
    pub fn new() -> Self {
        let mut board = board::Model::new();
        board.set(Board::start_position(EndChain));
        Self {
            board,
            socket: None,
        }
    }
}

pub fn update(_msg: Msg, _model: &mut Model, _orders: &mut impl Orders<Msg>) {}

pub fn view(model: &Model) -> Node<Msg> {
    let is_connecting = model
        .socket
        .as_ref()
        .map_or(true, |s| matches!(s.state(), web_socket::State::Connecting));

    div![
        C!["w-full  flex"],
        div![
            C!["w-2/3 lg:w-1/3 p-2"],
            board::view(&model.board, "w-full shadow").map_msg(|_| Msg::Board)
        ],
        div![
            C!["bg-gray-100 shadow flex-1 rounded-br"],
            if is_connecting {
                div![
                    C!["w-full h-full flex justify-center items-center font-bold text-gray-600"],
                    p!["Connecting..."]
                ]
            } else {
                Node::Empty
            }
        ]
    ]
}
