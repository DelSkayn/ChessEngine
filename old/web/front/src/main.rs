#![allow(non_snake_case)]
#![allow(dead_code)]

use log::info;
use sycamore::prelude::*;

mod app;
mod board;
mod components;
mod engine;
mod position;
mod user;
mod watch;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    info!("starting");

    sycamore::render(|cx| view! {cx, App {} });
}
