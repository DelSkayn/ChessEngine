use std::{collections::HashSet, time::Duration};

use common::engine::{self, Engine, UploadResponse};
use gloo_net::http;
use log::{debug, error, info};
use sycamore::{futures::spawn_local_scoped, prelude::*, rt::JsCast};
use web_sys::{Event, FormData, HtmlInputElement};

use crate::user::UserInfo;

async fn fetch_engine() -> Vec<Engine> {
    let resp = http::Request::new("/api/v1/engine")
        .method(http::Method::GET)
        .send()
        .await;

    let resp = match resp {
        Err(e) => {
            error!("error loading engines: {e}");
            return Vec::new();
        }
        Ok(x) => x,
    };

    let status = resp.status();
    if status != 200 {
        error!(
            "error loading engines: recieved status {status}, {}",
            resp.status_text()
        );
        return Vec::new();
    }

    let engines = resp.json::<Vec<Engine>>().await;

    match engines {
        Err(e) => {
            error!("error loading engines: {e}");
            Vec::new()
        }
        Ok(x) => x,
    }
}

async fn delete_engine(id: i32, token: String) {
    let body = serde_urlencoded::to_string(engine::DeleteReq { id }).unwrap();

    let resp = http::Request::new("/api/v1/engine")
        .method(http::Method::DELETE)
        .header("Authorization", &token)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await;

    let resp = match resp {
        Err(e) => {
            error!("error deleting engine: {e}");
            return;
        }
        Ok(x) => x,
    };

    let status = resp.status();
    if status != 200 {
        error!(
            "error deleting engine: recieved status {status}, {}",
            resp.status_text()
        );
        return;
    }

    let engines = resp.json::<engine::DeleteRes>().await;
    match engines {
        Err(e) => {
            error!("error deleteing engines: {e}");
            return;
        }
        Ok(x) => x,
    };
}

#[component]
pub fn EnginesUpload<G: Html>(cx: Scope<'_>) -> View<G> {
    let file_input = create_node_ref(cx);
    let engine_description = create_signal(cx, String::new());
    let modal_open = create_signal(cx, false);
    let is_uploading = create_signal(cx, false);
    let has_file = create_signal(cx, false);
    let error = create_signal::<Option<String>>(cx, None);
    let login_info = use_context::<Signal<UserInfo>>(cx);

    let upload_engine = move || {
        let file_input = file_input.get::<DomNode>();
        let file_input = file_input.unchecked_into::<HtmlInputElement>();
        let Some(file) = file_input.files().unwrap().get(0) else {
            return;
        };
        error.set(None);

        let info = login_info.get_untracked();
        let UserInfo::LoggedIn { ref token, .. } = *info else {
            return;
        };

        is_uploading.set(true);

        let data = FormData::new().unwrap();
        data.append_with_str("description", &engine_description.get())
            .unwrap();
        data.append_with_blob_and_filename("file", &file, file.name().as_str())
            .unwrap();

        let header_value = format!("Bearer {}", token);

        sycamore::futures::spawn_local_scoped(cx, async move {
            let resp = http::Request::new("/api/v1/engine")
                .header("Authorization", &header_value)
                .method(http::Method::POST)
                .body(data)
                .send()
                .await;

            is_uploading.set(false);

            let resp = match resp {
                Ok(x) => x,
                Err(e) => {
                    error.set(Some(format!("Failed to upload engine: {e}")));
                    return;
                }
            };
            if resp.status() != 200 {
                let text = resp.text().await;
                match text {
                    Ok(x) => error.set(Some(x)),
                    Err(x) => error.set(Some(format!("{x}"))),
                }
                return;
            }

            let data = resp.json::<UploadResponse>().await;

            let Ok(resp) = data else {
                error.set(Some("server returned an unexpected response".to_owned()));
                return;
            };
            match resp {
                UploadResponse::Ok => {
                    let ctx = use_context::<Signal<EngineContext>>(cx);
                    gloo_timers::future::sleep(Duration::from_secs(1)).await;

                    ctx.modify().loading = true;
                    modal_open.set(false);
                }
                UploadResponse::Err { error: e, .. } => {
                    error.set(Some(format!("Something went wrong: {e}")));
                }
            }
        })
    };

    let on_file_changed = |e: Event| {
        if let Some(files) = e
            .target()
            .and_then(|x| x.dyn_into::<HtmlInputElement>().ok())
            .and_then(|x| x.files())
        {
            has_file.set(files.length() != 0);
        } else {
            has_file.set(false);
        }
    };

    let upload_disabled = create_memo(cx, || !*has_file.get() || !login_info.get().is_logged_in());

    view! { cx,
        div(class="p-2"){
            label(for="upload-modal", class="btn w-full"){ "Add new" }
            input(type="checkbox",
                id="upload-modal",
                class="modal-toggle",
                bind:checked=modal_open
            )
            label(for="upload-modal", class="modal cursor-pointer"){
                div(class="modal-box"){
                    h3(class="font-bold text-lg"){ "Upload an chess engine" }
                    div(class="py-2"){
                        label(class="label", for="upload-engine-file"){
                            span(class="label-text", id="upload-engine-file"){ "Pick a engine executable" }
                        }
                        input(ref=file_input,type="file", class="file-input w-full", on:change=on_file_changed )
                    }
                    label(class="label", for="upload-engine-descr"){
                        span(class="label-text"){ "Engine description" }
                    }
                    textarea(class="textarea w-full textarea-bordered",
                        id="upload-engine-descr",
                        placeholder="Description",
                        bind:value=engine_description
                    )
                    button(class="btn btn-primary mt-2",disabled=*upload_disabled.get(), on:click=move |_| upload_engine()){ "Upload" }
                    ({
                        if let Some(x) = (*error.get()).clone(){
                            view!{cx,
                                div(class="alert alert-error shadow-lg mt-2"){
                                    div{
                                        svg(xmlns="http://www.w3.org/2000/svg",
                                            class="stroke-current flex-shrink-0 h-6 w-6",
                                            fill="none",
                                            viewBox="0 0 24 24"){
                                            path(stroke-linecap="round", stroke-width="2", d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z")
                                        }
                                        span{ (x.to_owned()) }

                                    }
                                }
                            }
                        }else{
                            view!(cx,)
                        }
                    })
                }
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct EngineContext {
    engines: Vec<Engine>,
    loading: bool,
}

#[component]
pub fn Engines<G: Html>(cx: Scope<'_>) -> View<G> {
    let context = create_signal(
        cx,
        EngineContext {
            engines: Vec::new(),
            loading: true,
        },
    );
    let context = provide_context_ref(cx, context);

    create_effect(cx, move || {
        if context.get().loading {
            debug!("reloading engines");
            spawn_local_scoped(cx, async {
                let engines = fetch_engine().await;
                context.set(EngineContext {
                    engines,
                    loading: false,
                });
            });
        }
    });

    let checked_engines = create_signal(cx, HashSet::<i32>::new());

    let login_info = use_context::<Signal<UserInfo>>(cx);

    let all_checked = create_memo(cx, || {
        let checked_engines = checked_engines.get_untracked();
        for engine in context.get().engines.iter() {
            if !checked_engines.contains(&engine.id) {
                return false;
            }
        }
        true
    });

    let is_checked = move |id: i32| checked_engines.get().contains(&id);

    let toggle_check = move |id: i32| {
        info!("checked {}", id);
        let mut checked = checked_engines.modify();
        if !checked.remove(&id) {
            checked.insert(id);
        };
    };

    let toggle_all = move || {
        if *all_checked.get() {
            checked_engines.set(
                context
                    .get_untracked()
                    .engines
                    .iter()
                    .map(|x| x.id)
                    .collect(),
            );
        } else {
            checked_engines.set(HashSet::new())
        }
    };

    let selected = create_signal::<Option<i32>>(cx, None);

    let select = move |id| {
        info!("select: {id}");
        selected.set(Some(id));
    };

    let on_click = move |_| {
        let login_info = login_info.get();
        let UserInfo::LoggedIn { ref token, .. } = *login_info else {
            return;
        };
        let header_value = format!("Bearer {}", token);

        let Some(id) = *selected.get() else { return };

        spawn_local_scoped(cx, async move {
            delete_engine(id, header_value).await;
            context.modify().loading = true;
        });
    };

    view! { cx,
        div(class="h-full w-full flex"){
            div(class="w-60 border-r-2 border-base-200 h-full"){
                (if login_info.get().is_logged_in(){
                    view!{ cx, EnginesUpload() }
                }else{
                    view!{ cx,}
                })
                table(class="table table-compact w-full table-auto"){
                    thead{
                        tr{
                            th(class="rounded-none"){
                                input(type="checkbox", checked=*all_checked.get(), class="checkbox checkbox-sm", on:mousedown= move |_| toggle_all()){}
                            }
                            th(class="rounded-none"){ "Name" }
                            th(class="rounded-none"){ "ELO" }
                        }
                    }
                    tbody(class="h-full overflow-y-scroll"){
                        Keyed(
                            iterable=context.map(cx,|x| x.engines.clone()),
                            view=move |cx,x| {

                                let class = move ||{
                                    let mut class = "hover".to_owned();
                                    if *selected.get() == Some(x.id){
                                        class += " selected";
                                    }
                                    class
                                };

                                view! { cx,
                                    tr(class=class(),on:click=move |_| select(x.id)){
                                        td {
                                            input(type="checkbox", checked=is_checked(x.id), class="checkbox checkbox-sm", on:click=move |_| toggle_check(x.id) ){}
                                        }
                                    td { (x.name) }
                                    td { (x.elo) }
                                    }
                                }
                            },
                            key=|x| x.id
                        )
                  }
            }
            }
            div(class="h-full flex-grow bg-base-200 p-2"){
                (if let Some(selected) = *selected.get(){
                    let descr = context.get()
                        .engines
                        .iter()
                        .find(|x| x.id == selected)
                        .unwrap()
                        .description
                        .clone()
                        .unwrap_or_else(String::new);

                    view!{cx,
                        div(class="flex flex-col"){
                            span{ (descr) }
                            div{
                                button(class="btn btn-error",
                                    on:click=on_click,
                                    disabled=!login_info.get().is_logged_in()){ "Delete" }
                            }
                        }
                    }
                }else{
                    view!{cx,
                    }
                })
            }
        }
    }
}
