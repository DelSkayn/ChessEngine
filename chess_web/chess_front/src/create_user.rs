use log::{error, info};
use seed::{attrs, div, fetch, h2, input, prelude::*, C};
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum Msg {
    Show,
    Hide,
    Username(String),
    Password(String),
    CreateUser,
}

#[derive(Serialize)]
struct CreateUserReq {
    username: String,
    password: String,
}

#[derive(Debug)]
pub struct Model {
    username: String,
    password: String,
    shown: bool,
}

impl Model {
    pub fn new() -> Self {
        Model {
            username: String::new(),
            password: String::new(),
            shown: false,
        }
    }
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Username(x) => model.username = x,
        Msg::Password(x) => model.password = x,
        Msg::Show => model.shown = true,
        Msg::Hide => model.shown = false,
        Msg::CreateUser => {
            if model.username.is_empty() || model.password.is_empty() {
                orders.skip();
                return;
            }
            orders.skip().perform_cmd({
                let login_data = CreateUserReq {
                    username: model.username.clone(),
                    password: model.password.clone(),
                };
                async move {
                    attempt_login(login_data)
                        .await
                        .map_err(|e| {
                            error!("error {e:?}");
                            e
                        })
                        .ok();
                }
            });
        }
    }
}

async fn attempt_login(create_user_data: CreateUserReq) -> fetch::Result<()> {
    let res = Request::new("/api/v1/user")
        .method(Method::Post)
        .body(
            serde_urlencoded::to_string(&create_user_data)
                .unwrap()
                .into(),
        )
        .header(Header::content_type(
            "application/x-www-form-urlencoded;charset=UTF-8",
        ))
        .fetch()
        .await?
        .check_status()?
        .text()
        .await?;

    info!("response: {res}");

    Ok(())
}

pub fn view(model: &Model) -> Node<Msg> {
    if model.shown {
        div![
            C!["fixed left-0 top-0 right-0 bottom-0 bg-gray-600 bg-opacity-50 w-full h-full flex flex-col mt-2 items-center justify-center"],
            //ev(Ev::Click, |_| Msg::Hide),
            div![
                C!["flex flex-col w-1/2 bg-gray-100 p-4 rounded "],
                h2!["Create an account"],
                input![
                    attrs! {
                        At::Type => "text",
                        At::Placeholder => "Username",
                        At::Value => model.username,
                    },
                    input_ev(Ev::Input, Msg::Username),
                    C!["p-2 px-3 shadow-inner rounded my-1 border border-gray-400"],
                ],
                input![
                    attrs! {
                        At::Type => "password",
                        At::Placeholder => "Password",
                        At::Value => model.password,
                    },
                    input_ev(Ev::Input, Msg::Password),
                    C!["p-2 px-3 shadow-inner rounded my-1 border border-gray-400"],
                ],
                input![
                    attrs! {
                        At::Type => "button",
                        At::Value => "Create account",
                        At::Disabled => (model.username.is_empty() || model.password.is_empty()).as_at_value()
                    },
                    C!["text-lg transition-all p-1 shadow rounded my-1 bg-green-400 border border-green-400 text-gray-100 hover:bg-green-500 disabled:bg-gray-400 disabled:border-gray-400 disabled:text-gray-200"],
                    ev(Ev::Click, |_| Msg::CreateUser),
                ]
            ]
        ]
    } else {
        Node::Empty
    }
}
