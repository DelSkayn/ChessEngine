use crate::Global;
use seed::{div, p, prelude::*, C, IF};
use serde::Deserialize;

mod list;
mod upload;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tab {
    Upload,
    List,
}

#[derive(Debug)]
pub struct Model {
    upload: upload::Model,
    list: list::Model,
    active_tab: Tab,
}

impl Model {
    pub fn new(orders: &mut impl Orders<Msg>) -> Self {
        Model {
            upload: upload::Model::new(),
            list: list::Model::new(&mut orders.proxy(Msg::List)),
            active_tab: Tab::List,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum UploadResponse {
    Ok { ok: bool },
    Err { error: String },
}

#[derive(Debug)]
pub enum Msg {
    Upload(upload::Msg),
    List(list::Msg),
    SwitchTab(Tab),
}

pub fn update(msg: Msg, model: &mut Model, global: &mut Global, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Upload(x) => {
            upload::update(x, &mut model.upload, global, &mut orders.proxy(Msg::Upload))
        }
        Msg::List(x) => list::update(x, &mut model.list, &mut orders.proxy(Msg::List)),
        Msg::SwitchTab(tab) => {
            if tab == Tab::List {
                list::load_engines(&mut orders.proxy(Msg::List));
            }
            model.active_tab = tab;
        }
    }
}

pub fn view(model: &Model, global: &Global) -> Node<Msg> {
    let inner = match model.active_tab {
        Tab::Upload => upload::view(&model.upload, global).map_msg(Msg::Upload),
        Tab::List => list::view(&model.list).map_msg(Msg::List),
    };

    let tab_button = |tab: Tab, text: &str, active: Tab| {
        div![
            ev(Ev::Click, move |_| Msg::SwitchTab(tab)),
            C!["px-2 py-1 font-bold text-gray-600 cursor-pointer"],
            IF!(tab == active => C!["text-green-600 "]),
            p![text]
        ]
    };

    div![
        C!["w-full"],
        div![
            C!["flex space-x-2 px-1 border-b divide-x bg-gray-200"],
            tab_button(Tab::List, "List", model.active_tab),
            tab_button(Tab::Upload, "Upload", model.active_tab),
        ],
        inner
    ]
}
