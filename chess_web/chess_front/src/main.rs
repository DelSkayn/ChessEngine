#![allow(dead_code)]

use chess_core::{board::EndChain, Board};
use log::info;
use seed::{attrs, div, h1, input, prelude::*, C};

mod board;
mod create_user;
mod engine;
mod file_upload;
mod login;
//mod util;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    info!("Starting chess web!");
    //dioxus::web::launch_with_props(app, (), |c| c.rootname("root"))
    App::start("root", init, update, view);
}

#[derive(Debug)]
pub enum Msg {
    Login(login::Msg),
    Engine(engine::Msg),
    CreateUser(create_user::Msg),
    ShowCreateUser,
}

pub struct Model {
    board: board::Model,
    login: login::Model,
    engine: engine::Model,
    create_user: create_user::Model,
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let mut board = board::Model::new();
    board.set(Board::start_position(EndChain));

    Model {
        board,
        login: login::Model::new(),
        engine: engine::Model::new(),
        create_user: create_user::Model::new(),
    }
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    info!("{:?}", msg);
    match msg {
        Msg::Login(msg) => login::update(msg, &mut model.login, &mut orders.proxy(Msg::Login)),
        Msg::Engine(msg) => {
            engine::update(msg, &mut model.engine, &mut orders.proxy(Msg::Engine));
        }
        Msg::CreateUser(msg) => {
            create_user::update(
                msg,
                &mut model.create_user,
                &mut orders.proxy(Msg::CreateUser),
            );
        }
        Msg::ShowCreateUser => create_user::update(
            create_user::Msg::Show,
            &mut model.create_user,
            &mut orders.proxy(Msg::CreateUser),
        ),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        C!["w-full h-full flex items-center justify-center bg-gray-200"],
        div![
            C!["flex flex-col p-4 w-1/3 shadow-xl rounded-lg bg-gray-100"],
            h1![C!["italic text-gray-400 mb-2 select-none"], "Chess Web"],
            board::view(&model.board, "w-full"),
            login::view(&model.login).map_msg(Msg::Login),
            engine::view(&model.engine).map_msg(Msg::Engine),
            create_user::view(&model.create_user).map_msg(Msg::CreateUser),
            input![
                attrs! {
                    At::Type => "button",
                    At::Value => "Create a new account!",
                },
                C!["text-lg transition-all p-1 shadow rounded my-1 bg-green-400 border border-green-400 text-gray-100 hover:bg-green-500 disabled:bg-gray-400 disabled:border-gray-400 disabled:text-gray-200"],
                ev(Ev::Click, |_| Msg::ShowCreateUser),
            ]
        ],
    ]
}
