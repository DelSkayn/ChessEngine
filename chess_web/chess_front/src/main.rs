#![allow(dead_code)]

use log::info;
use seed::{div, p, prelude::*, C, IF};

mod board;
mod components;
mod tabs;
mod top_bar;

pub use tabs::*;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    info!("Starting chess web!");
    //dioxus::web::launch_with_props(app, (), |c| c.rootname("root"))
    App::start("root", init, update, view);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Watch,
    Engines,
    Games,
}

#[derive(Debug)]
pub enum Msg {
    TopBar(top_bar::Msg),
    Empty,
    SwitchTab(Tab),
    Watch(watch::Msg),
    Engine(engine::Msg),
    //Login(login::Msg),
    //CreateUser(create_user::Msg),
    //ShowCreateUser,
}

pub struct Global {
    user_token: Option<String>,
}

pub struct Model {
    global: Global,
    topbar: top_bar::Model,
    tab_watch: watch::Model,
    tab_engine: engine::Model,
    active_tab: Tab,
    //login: login::Model,
    //engine: engine::Model,
    //create_user: create_user::Model,
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        global: Global { user_token: None },
        topbar: top_bar::Model::new(),
        tab_watch: watch::Model::new(),
        tab_engine: engine::Model::new(),
        active_tab: Tab::Watch,
        //login: login::Model::new(),
        //engine: engine::Model::new(),
        //create_user: create_user::Model::new(),
    }
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    info!("{:?}", msg);
    match msg {
        Msg::TopBar(msg) => {
            top_bar::update(
                msg,
                &mut model.topbar,
                &mut model.global,
                &mut orders.proxy(Msg::TopBar),
            );
        }
        Msg::Watch(x) => watch::update(x, &mut model.tab_watch, &mut orders.proxy(Msg::Watch)),
        Msg::Engine(x) => engine::update(
            x,
            &mut model.tab_engine,
            &mut model.global,
            &mut orders.proxy(Msg::Engine),
        ),
        Msg::Empty => {
            orders.skip();
        }
        Msg::SwitchTab(tab) => {
            model.active_tab = tab;
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    let menu_button = |text: &str, active: bool, tab: Tab| {
        div![
            ev(Ev::Click,move |_| Msg::SwitchTab(tab)),
            C!["font-bold text-gray-600 px-4 h-full flex items-center hover:bg-gray-300 cursor-pointer"],
            IF!(active => C!["text-green-600"]),
            p![text],
        ]
    };

    let tab = match model.active_tab {
        Tab::Watch => watch::view(&model.tab_watch).map_msg(Msg::Watch),
        Tab::Engines => engine::view(&model.tab_engine, &model.global).map_msg(Msg::Engine),
        Tab::Games => Node::Empty,
    };

    div![
        C!["w-full h-full bg-gray-200 overflow-scroll flex flex-col items-center"],
        div![
            C!["mt-16 container flex flex-col"],
            div![
                C!["h-10 bg-gray-200 rounded-t shadow flex items-center overflow-hidden select-none"],
                menu_button("Watch", model.active_tab == Tab::Watch, Tab::Watch),
                menu_button("Engines", model.active_tab == Tab::Engines, Tab::Engines),
                menu_button("Games", model.active_tab == Tab::Games, Tab::Games),
            ],
            div![
                C!["bg-gray-100 shadow rounded-b overflow-hidden"],
                tab
            ]
        ],
        top_bar::view(&model.topbar, &model.global).map_msg(Msg::TopBar)
    ]
}
