use sycamore::prelude::*;

#[component(inline_props)]
pub fn ErrorAlert<'a, G: Html>(cx: Scope<'a>, error: &'a ReadSignal<Option<String>>) -> View<G> {
    view! {cx, (if let Some(x) = (*error.get()).clone() {
            view!{cx,
            div(class="alert alert-error shadow-lg mt-2"){
                div{
                    svg(xmlns="http://www.w3.org/2000/svg",
                        class="stroke-current flex-shrink-0 h-6 w-6",
                        fill="none",
                        viewBox="0 0 24 24"){
                        path(stroke-linecap="round", stroke-width="2", d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z")
                    }
                    span{ (x) }

                }
            }
            }
    } else {
        view!{cx, }
    })}
}
