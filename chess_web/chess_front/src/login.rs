use log::info;
use seed::{attrs, div, fetch, input, prelude::*, C};
use serde::{Deserialize, Serialize};
use web_sys::{Event, KeyboardEvent};

#[derive(Serialize)]
struct LoginData {
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
//#[serde(untagged)]
pub enum LoginResp {
    Err { error: String },
    Ok { token: String },
}

pub struct Model {
    username: String,
    password: String,
}

#[derive(Debug)]
pub enum Msg {
    Login,
    LoginResult(fetch::Result<LoginResp>),

    Username(String),
    Password(String),
}

impl Model {
    pub fn new() -> Self {
        Model {
            username: String::new(),
            password: String::new(),
        }
    }
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Username(x) => model.username = x,
        Msg::Password(x) => model.password = x,
        Msg::Login => {
            if model.username.is_empty() || model.password.is_empty() {
                orders.skip();
                return;
            }
            orders.skip().perform_cmd({
                let login_data = LoginData {
                    username: model.username.clone(),
                    password: model.password.clone(),
                };
                async move { Msg::LoginResult(attempt_login(login_data).await) }
            });
        }
        Msg::LoginResult(x) => {
            info!("{:?}", x);
        }
    }
}

async fn attempt_login(login_data: LoginData) -> fetch::Result<LoginResp> {
    Request::new("/api/v1/user/login")
        .method(Method::Post)
        .body(serde_urlencoded::to_string(&login_data).unwrap().into())
        .header(Header::content_type(
            "application/x-www-form-urlencoded;charset=UTF-8",
        ))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

fn event_handler(event: Event) -> Option<Msg> {
    if let Some(e) = event.dyn_ref::<KeyboardEvent>() {
        if e.key_code() == 10 || e.key_code() == 13 {
            return Some(Msg::Login);
        }
    }
    None
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        C!["flex  flex-col mt-2"],
        input![
            attrs! {
                At::Type => "text",
                At::Placeholder => "Username",
                At::Value => model.username,
            },
            input_ev(Ev::Input, Msg::Username),
            ev(Ev::KeyPress, event_handler),
            C!["p-2 px-3 shadow-inner rounded my-1 border border-gray-400"],
        ],
        input![
            attrs! {
                At::Type => "password",
                At::Placeholder => "Password",
                At::Value => model.password,
            },
            input_ev(Ev::Input, Msg::Password),
            ev(Ev::KeyPress, event_handler),
            C!["p-2 px-3 shadow-inner rounded my-1 border border-gray-400"],
        ],
        input![
            attrs! {
                At::Type => "button",
                At::Value => "Login",
                At::Disabled => (model.username.is_empty() || model.password.is_empty()).as_at_value()
            },
            C!["text-lg transition-all p-1 shadow rounded my-1 bg-green-400 border border-green-400 text-gray-100 hover:bg-green-500 disabled:bg-gray-400 disabled:border-gray-400 disabled:text-gray-200"],
            ev(Ev::Click, |_| Msg::Login),
        ]
    ]
}
