use crate::components::{icon, Icon};
use futures::FutureExt;
use log::info;
use seed::{
    attrs, circle, div, input, p, prelude::*, rect, svg, table, tbody, td, th, thead, tr, util, C,
    IF,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Engine {
    name: String,
    description: Option<String>,
    elo: f32,
    games_played: u32,
}

#[derive(Debug)]
pub struct Model {
    engines: Vec<Engine>,
    error: bool,
    confirm_delete: Option<u32>,
}

impl Model {
    pub fn new(orders: &mut impl Orders<Msg>) -> Self {
        load_engines(orders);
        Self {
            engines: Vec::new(),
            error: false,
            confirm_delete: Some(0),
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    Load(Result<Vec<Engine>, ()>),
    Reload,
    Edit(u32),
    Delete(u32),
    DeleteConfirm,
    DeleteClear,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Load(Ok(x)) => {
            model.engines = x;
        }
        Msg::Reload => {
            load_engines(orders);
            orders.skip();
        }
        Msg::Load(Err(_)) => model.error = true,
        Msg::Edit(_) => {}
        Msg::Delete(x) => {
            model.confirm_delete = Some(x);
        }
        Msg::DeleteConfirm => {
            model.confirm_delete = None;
        }
        Msg::DeleteClear => {
            model.confirm_delete = None;
        }
    }
}

async fn fetch_engine() -> Result<Vec<Engine>, ()> {
    Request::new("/api/v1/engine")
        .method(Method::Get)
        .fetch()
        .await
        .map_err(|_| ())?
        .check_status()
        .map_err(|_| ())?
        .json()
        .await
        .map_err(|x| {
            util::error(x);
        })
}

pub fn load_engines(orders: &mut impl Orders<Msg>) {
    orders.perform_cmd(async { fetch_engine().map(Msg::Load).await });
}

pub fn view(model: &Model) -> Node<Msg> {
    let content = model
        .engines
        .iter()
        .enumerate()
        .map(|(id, x)| {
            let delete = if Some(id as u32) == model.confirm_delete {
                div![
                    C!["right-9 -top-2 text-gray-600 bg-gray-100 absolute rounded shadow w-48 flex px-2 py-1 space-x-1 border border-gray-300"],
                    p!["Are you sure?"],
                    input![
                        C!["transition-all px-1 shadow rounded  bg-red-400 border border-red-400 text-gray-100 hover:bg-red-500"],
                        attrs! {
                            At::Type => "button",
                            At::Value => "Yes",
                        },
                        ev(Ev::Click, |_| Msg::DeleteConfirm)
                    ],
                    input![
                        C!["transition-all px-1 shadow rounded  bg-gray-400 border border-gray-400 text-gray-100 hover:bg-gray-500"],
                        attrs! {
                            At::Type => "button",
                            At::Value => "No",
                        },
                        ev(Ev::Click, |_| {
                            Msg::DeleteClear
                        })
                    ],
                    ev(Ev::Click, |x|{
                        x.stop_propagation();
                    })
                ]
            } else {
                Node::Empty
            };

            tr![
                C!["even:bg-gray-200"],
                td![
                    C!["text-left py-2 px-3 font-bold w-4"],
                    (id + 1).to_string()
                ],
                td![C!["text-left py-2 px-3"], &x.name],
                td![C!["text-left py-2 px-3"], format!("{}", x.elo)],
                td![
                    C!["text-left py-2 px-3 truncate italic"],
                    x.description.as_deref().unwrap_or("")
                ],
                td![
                    C!["px-3 text-center"],
                    div![
                        C!["flex flex-shrink-0 space-x-1 relative"],
                        div![
                            C!["hover:text-green-600 cursor-pointer"],
                            ev(Ev::Click, move |_| Msg::Edit(id as u32)),
                            icon(Icon::Pencil, "h-5 w-5"),
                        ],
                        div![
                            C!["hover:text-red-600 cursor-pointer"],
                            IF!(Some(id as u32) == model.confirm_delete => C!["text-red-600"]),
                            ev(Ev::Click, move |_| Msg::Delete(id as u32)),
                            icon(Icon::Trash, "h-5 w-5"),
                        ],
                            delete,
                    ]
                ]
            ]
        })
        .collect::<Vec<_>>();

    div![
        C!["px-2 py-4"],
        table![
            C!["table-auto w-full rounded overflow-hidden border-collapse border border-slate-300 shadow"],
            thead![
                C!["font-bold text-gray-100 bg-gray-500"],
                tr![
                    th![C!["text-left py-2 px-3"], "#"],
                    th![C!["text-left py-2 px-3"], "Name"],
                    th![C!["text-left py-2 px-3"], "Elo"],
                    th![C!["text-left py-2 px-3"], "Description"],
                    th![
                        C!["pl-5"],
                        div![
                            C!["hover:text-green-400 cursor-pointer"],
                            ev(Ev::Click,|_| Msg::Reload),
                            icon(Icon::Refresh, "h-6 w-6")
                        ]
                    ]
                ]
            ],
            tbody![C!["text-gray-700"], content]
        ]
    ]
}
