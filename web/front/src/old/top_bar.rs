use seed::{div, h3, p, prelude::*, C};

use crate::Global;

mod create_user;
mod login;

#[derive(Debug)]
pub enum Msg {
    Hide,
    ShowLogin,
    ShowRegister,
    Logout,
    Login(login::Msg),
    Register(create_user::Msg),
}

#[derive(Debug)]
pub struct Model {
    login_shown: bool,
    register_shown: bool,
    login: login::Model,
    create_user: create_user::Model,
}

pub fn update(msg: Msg, model: &mut Model, global: &mut Global, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Hide => {
            model.login_shown = false;
            model.register_shown = false;
        }
        Msg::ShowLogin => {
            model.login_shown = !model.login_shown;
            model.register_shown = false;
        }
        Msg::ShowRegister => {
            model.login_shown = false;
            model.register_shown = !model.register_shown;
        }
        Msg::Logout => {
            global.user_token = None;
        }
        Msg::Login(x) => {
            if let login::Msg::LoginResult(Ok(_)) = x {
                model.login_shown = false;
            }
            login::update(x, &mut model.login, global, &mut orders.proxy(Msg::Login))
        }
        Msg::Register(x) => {
            create_user::update(x, &mut model.create_user, &mut orders.proxy(Msg::Register))
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Model {
            login_shown: false,
            register_shown: false,
            login: login::Model::new(),
            create_user: create_user::Model::new(),
        }
    }
}

pub fn view(model: &Model, global: &Global) -> Node<Msg> {
    let login_button = if global.user_token.is_none() {
        div![
            ev(Ev::Click,|_| Msg::ShowLogin),
            C!["px-3 py-1 relative rounded-full text-green-600 font-bold hover:bg-gray-300 cursor-pointer select-none"],
            p!["Login"],
            if model.login_shown{
                div![
                    ev(Ev::Click,|ev|{
                        ev.stop_propagation();
                    }),
                    div![
                        C!["absolute p-2 w-64 shadow rounded right-0 top-12 bg-gray-100 z-50"],
                        h3!["Login"],
                        login::view(&model.login).map_msg(Msg::Login),
                    ],
                ]
            }else{
                Node::Empty
            }
            ]
    } else {
        div![
            ev(Ev::Click,|_| Msg::Logout),
            C!["px-3 py-1 rounded-full text-red-600 font-bold hover:bg-gray-300 cursor-pointer select-none"],
            p!["Logout"]
        ]
    };

    let create_user = if global.user_token.is_none() {
        div![
            ev(Ev::Click,|_| Msg::ShowRegister),
            C!["px-3 py-1 relative rounded-full text-green-600 font-bold hover:bg-gray-300 cursor-pointer select-none"],
            p!["Register"],
            if model.register_shown{
                div![
                    ev(Ev::Click,|ev|{
                        ev.stop_propagation();
                    }),
                    div![
                        C!["absolute p-2 w-64 shadow rounded right-0 top-10 bg-gray-100 z-50"],
                        h3!["Register"],
                        create_user::view(&model.create_user).map_msg(Msg::Register),
                    ],
                ]
            }else{
                Node::Empty
            }
            ]
    } else {
        Node::Empty
    };

    div![
        C!["w-full h-12 fixed top-0 flex justify-between"],
        div![
            C!["flex justify-center items-center w-24"],
            h3![C!["font-bold italic text-gray-600"], "Chess"]
        ],
        div![
            C!["flex justify-center items-center space-x-4 px-2"],
            login_button,
            create_user
        ],
    ]
}
