use crate::{position::Position, watch::Watch};

use sycamore::prelude::*;

use crate::{
    engine::Engines,
    user::{UserInfo, UserMenu},
};

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum ActivePage {
    Engines,
    Watch,
    Position,
}

#[component]
pub fn Loading<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div(class="w-full h-full flex justify-center items-center"){
                    span(class="loader")
        }
    }
}

#[component(inline_props)]
pub fn SidebarItem<G: Html>(cx: Scope, name: &'static str, kind: ActivePage) -> View<G> {
    let active_page = use_context::<Signal<ActivePage>>(cx);

    let class = move || {
        if *active_page.get() == kind {
            "active"
        } else {
            ""
        }
    };

    view! { cx,
        li{ a(on:click=move |_| { active_page.set(kind) },class=class()){ (name) }}
    }
}

#[component]
pub fn App<G: Html>(cx: Scope) -> View<G> {
    let user_info = create_signal(cx, UserInfo::NotLoggedIn);
    provide_context_ref(cx, user_info);
    let active_page = create_signal(cx, ActivePage::Engines);
    provide_context_ref(cx, active_page);

    view! { cx,
        div(class="w-full h-full flex flex-col"){
            div(class="navbar w-full h-14 border-base-200 border-b-2 flex justify-between items-center"){
                h1(class="text-3xl font-bold italic px-4"){ "Chess" }
                UserMenu {}
            }
            div(class="grow flex"){
                div(class="w-60 border-base-200 border-r-2"){
                    ul(class="menu"){
                        SidebarItem(name="Watch",kind=ActivePage::Watch)
                        SidebarItem(name="Engine",kind=ActivePage::Engines)
                        SidebarItem(name="Position",kind=ActivePage::Position)
                    }
                }
                div(class="grow flex justify-center"){
                    (match *active_page.get(){
                        ActivePage::Engines => Engines(cx),
                        ActivePage::Watch => Watch(cx),
                        ActivePage::Position => Position(cx)
                    })
                }
            }
        }
    }
}
