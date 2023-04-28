use gloo_net::http;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub enum UserInfo {
    NotLoggedIn,
    LoggedIn { name: String, token: String },
}

impl UserInfo {
    pub fn is_logged_in(&self) -> bool {
        match *self {
            UserInfo::NotLoggedIn => false,
            UserInfo::LoggedIn { .. } => true,
        }
    }

    pub fn name(&self) -> Option<String> {
        match *self {
            UserInfo::NotLoggedIn => None,
            UserInfo::LoggedIn { ref name, .. } => Some(name.clone()),
        }
    }
}

#[derive(Serialize)]
struct LoginReq<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize, Debug)]
//#[serde(untagged)]
pub enum LoginResp {
    Err { error: String },
    Ok { token: String },
}

async fn attemp_login(
    username: &Signal<String>,
    password: &Signal<String>,
    info: &Signal<UserInfo>,
) {
    info!("trying to login");
    let data = LoginReq {
        username: &*username.get(),
        password: &password.get(),
    };

    let resp = http::Request::new("/api/v1/user/login")
        .method(http::Method::POST)
        .body(serde_urlencoded::to_string(&data).unwrap())
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded;charset=UTF-8",
        )
        .send()
        .await;

    match resp {
        Ok(x) => match x.json::<LoginResp>().await {
            Ok(LoginResp::Ok { token }) => info.set(UserInfo::LoggedIn {
                name: (*username.get()).clone(),
                token,
            }),
            Ok(LoginResp::Err { error }) => {
                error!("error logging in: {}", error);
            }
            Err(e) => {
                error!("error logging in: {}", e)
            }
        },
        Err(e) => {
            error!("error logging in: {}", e)
        }
    }
}

#[component]
pub fn UserMenu<G: Html>(cx: Scope) -> View<G> {
    let userinfo = use_context::<Signal<UserInfo>>(cx);

    view! { cx,
        (if let Some(name) = userinfo.get().name(){
            view!{ cx, h4(class="font-bold text-primary px-4"){ (format!("Hello {}",name)) }}
        }else{
            view!{ cx,
                div(class="px-4 flex space-x-2"){
                    LoginButton {}
                    RegisterButton {}
                }
            }
        })
    }
}

#[component]
pub fn LoginButton<G: Html>(cx: Scope) -> View<G> {
    let username = create_signal(cx, String::new());
    let password = create_signal(cx, String::new());

    let userinfo = use_context::<Signal<UserInfo>>(cx);

    let is_disabled = || username.get().is_empty() || password.get().is_empty();

    let login = move || {
        info!("called");
        userinfo.track();
        spawn_local_scoped(cx, attemp_login(username, password, userinfo))
    };

    view! {cx,
        div(class="dropdown dropdown-end"){
            label(tabindex=0, class="btn"){"Login"}
            div(tabindex=0, class="dropdown-content rounded p-2 w-52 bg-base-200 mt-4 rounded-xl"){
                div(tabindex=0, class="form-control"){
                    label(class="label"){
                        span(class="label-text"){"Username"}
                    }
                    input(type="text", placeholder="Enter a username", class="input input-bordered", bind:value=username){
                    }
                }
                div(tabindex=0, class="form-control"){
                    label(class="label"){
                        span(class="label-text"){"Password"}
                    }
                    input(type="password", placeholder="Enter a password", class="input input-bordered", bind:value=password){
                    }
                }
                button(class="btn btn-primary w-full mt-4",disabled=is_disabled(), on:click=move |_|{login()}){ "Login" }
            }
        }
    }
}

#[component]
pub fn RegisterButton<G: Html>(cx: Scope) -> View<G> {
    view! {cx,
        div(class="dropdown dropdown-end"){
            label(tabindex=0, class="btn"){"Register"}
            div(tabindex=0, class="dropdown-content rounded p-2 w-52 bg-base-200 mt-4 rounded-xl"){
                div(tabindex=0, class="form-control"){
                    label(class="label"){
                        span(class="label-text"){"Username"}
                    }
                    input(type="text", placeholder="Enter a username", class="input input-bordered"){
                    }
                }
                div(tabindex=0, class="form-control"){
                    label(class="label"){
                        span(class="label-text"){"Password"}
                    }
                    input(type="password", placeholder="Enter a password", class="input input-bordered"){
                    }
                }
                button(class="btn btn-primary w-full mt-4"){ "Register" }
            }
        }
    }
}
